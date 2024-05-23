#![allow(clippy::too_many_arguments)]
use std::sync::Arc;

use hyper::service::service_fn;
use hyper_rustls::TlsAcceptor;
use hyper_util::rt::TokioExecutor;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde::de::DeserializeOwned;
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::cli::CLIError;
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest, GraphQLRequestLike};
use crate::core::http::{handle_request, Request};

pub async fn start_http_2(
    sc: Arc<ServerConfig>,
    cert: Vec<CertificateDer<'static>>,
    key: Arc<PrivateKeyDer<'static>>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let acceptor = TlsAcceptor::builder()
        .with_single_cert(cert, key.clone_key())?
        .with_http2_alpn()
        .with_acceptor(listener);

    let mut builder = hyper::server::conn::http2::Builder::new(TokioExecutor::new());


    let mut ty: impl GraphQLRequestLike + DeserializeOwned = GraphQLRequest;

    if sc.blueprint.server.enable_batch_requests {
        ty = GraphQLBatchRequest;
    };

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }


    loop {
        let (stream, _) = acceptor.;
        let connection = builder
            .serve_connection(
                stream,
                service_fn(move |req| {
                    async move {
                        let req = Request::from_hyper(req).await?;
                        handle_request::<ty>(req, sc.app_ctx.clone()).await
                    }
                }),
            )
            .with_upgrades();
        tokio::spawn(async move {
            if let Err(err) = connection.await {
                println!("Error serving HTTP connection: {err:?}");
            }
        });
    }

    let builder = Server::builder(acceptor).http2_only(true);

    super::log_launch(sc.as_ref());

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }

    let server: std::prelude::v1::Result<(), hyper::Error> =
        if sc.blueprint.server.enable_batch_requests {
            builder.serve(make_svc_batch_req).await
        } else {
            builder.serve(make_svc_single_req).await
        };

    let result = server.map_err(CLIError::from);

    Ok(result?)
}
