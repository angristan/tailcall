use std::sync::RwLock;

use trc::SharedTrc;

use super::verification::Verification;
use super::verify::{AuthVerifier, Verify};
use crate::blueprint::Auth;
use crate::http::RequestContext;

#[derive(Default)]
pub struct GlobalAuthContext {
    verifier: Option<AuthVerifier>,
}

#[derive(Default)]
pub struct AuthContext {
    auth_result: RwLock<Option<Verification>>,
    global_ctx: SharedTrc<GlobalAuthContext>,
}

impl GlobalAuthContext {
    // TODO: it could be better to return async_graphql::Error to make it more
    // graphql way with additional info. But this actually requires rewrites to
    // expression to work with that type since otherwise any additional info
    // will be lost during conversion to anyhow::Error
    async fn validate(&self, request: &RequestContext) -> Verification {
        if let Some(verifier) = self.verifier.as_ref() {
            verifier.verify(request).await
        } else {
            Verification::succeed()
        }
    }
}

impl GlobalAuthContext {
    pub fn new(auth: Option<Auth>) -> Self {
        Self { verifier: auth.map(AuthVerifier::from) }
    }
}

impl AuthContext {
    pub async fn validate(&self, request: &RequestContext) -> Verification {
        if let Some(result) = self.auth_result.read().unwrap().as_ref() {
            return result.clone();
        }

        let result = self.global_ctx.validate(request).await;

        self.auth_result.write().unwrap().replace(result.clone());

        result
    }
}

impl From<&SharedTrc<GlobalAuthContext>> for AuthContext {
    fn from(global_ctx: &SharedTrc<GlobalAuthContext>) -> Self {
        Self {
            global_ctx: global_ctx.clone(),
            auth_result: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::basic::tests::{create_basic_auth_request, HTPASSWD_TEST};
    use crate::auth::basic::BasicVerifier;
    use crate::auth::error::Error;
    use crate::auth::jwt::jwt_verify::tests::{create_jwt_auth_request, JWT_VALID_TOKEN_WITH_KID};
    use crate::auth::jwt::jwt_verify::JwtVerifier;
    use crate::auth::verify::Verifier;
    use crate::blueprint;

    #[tokio::test]
    async fn validate_request_missing_credentials() {
        let auth_context = setup_auth_context().await;
        let validation = auth_context.validate(&RequestContext::default()).await;
        assert_eq!(validation, Verification::fail(Error::Missing));
    }

    #[tokio::test]
    async fn validate_request_basic_auth_wrong_password() {
        let auth_context = setup_auth_context().await;
        let validation = auth_context
            .validate(&create_basic_auth_request("testuser1", "wrong-password"))
            .await;
        assert_eq!(validation, Verification::fail(Error::Invalid));
    }

    #[tokio::test]
    async fn validate_request_basic_auth_correct_password() {
        let auth_context = setup_auth_context().await;
        let validation = auth_context
            .validate(&create_basic_auth_request("testuser1", "password123"))
            .await;
        assert_eq!(validation, Verification::succeed());
    }

    #[tokio::test]
    async fn validate_request_jwt_auth_valid_token() {
        let auth_context = setup_auth_context().await;
        let validation = auth_context
            .validate(&create_jwt_auth_request(JWT_VALID_TOKEN_WITH_KID))
            .await;
        assert_eq!(validation, Verification::succeed());
    }

    // Helper function for setting up the auth context
    async fn setup_auth_context() -> GlobalAuthContext {
        let basic_provider =
            BasicVerifier::new(blueprint::Basic { htpasswd: HTPASSWD_TEST.to_owned() });
        let jwt_options = blueprint::Jwt::test_value();
        let jwt_provider = JwtVerifier::new(jwt_options);

        GlobalAuthContext {
            verifier: Some(AuthVerifier::Or(
                AuthVerifier::Single(Verifier::Basic(basic_provider)).into(),
                AuthVerifier::Single(Verifier::Jwt(jwt_provider)).into(),
            )),
        }
    }
}
