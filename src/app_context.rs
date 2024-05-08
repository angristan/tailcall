use async_graphql::dynamic::{self, DynamicRequest};
use async_graphql::Response;
use trc::SharedTrc;

use crate::auth::context::GlobalAuthContext;
use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition, SchemaModifiers};
use crate::data_loader::DataLoader;
use crate::graphql::GraphqlDataLoader;
use crate::grpc;
use crate::grpc::data_loader::GrpcDataLoader;
use crate::http::{DataLoaderRequest, HttpDataLoader};
use crate::lambda::{DataLoaderId, Expression, IO};
use crate::rest::{Checked, EndpointSet};
use crate::runtime::TargetRuntime;

pub struct AppContext {
    pub schema: dynamic::Schema,
    pub runtime: TargetRuntime,
    pub blueprint: SharedTrc<Blueprint>,
    pub http_data_loaders: SharedTrc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
    pub gql_data_loaders: SharedTrc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
    pub grpc_data_loaders: SharedTrc<Vec<DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>>>,
    pub endpoints: EndpointSet<Checked>,
    pub auth_ctx: SharedTrc<GlobalAuthContext>,
}

impl AppContext {
    pub fn new(
        mut blueprint: Blueprint,
        runtime: TargetRuntime,
        endpoints: EndpointSet<Checked>,
    ) -> Self {
        let mut http_data_loaders = vec![];
        let mut gql_data_loaders = vec![];
        let mut grpc_data_loaders = vec![];

        for def in blueprint.definitions.iter_mut() {
            if let Definition::Object(def) = def {
                for field in &mut def.fields {
                    let of_type = field.of_type.clone();
                    let upstream_batch = &blueprint.upstream.batch;
                    field.map_expr(|expr| {
                        expr.modify(|expr| match expr {
                            Expression::IO(io) => match io {
                                IO::Http { req_template, group_by, .. } => {
                                    let data_loader = HttpDataLoader::new(
                                        runtime.clone(),
                                        group_by.clone(),
                                        matches!(of_type, ListType { .. }),
                                    )
                                    .to_data_loader(upstream_batch.clone().unwrap_or_default());

                                    let result = Some(Expression::IO(IO::Http {
                                        req_template: req_template.clone(),
                                        group_by: group_by.clone(),
                                        dl_id: Some(DataLoaderId(http_data_loaders.len())),
                                    }));

                                    http_data_loaders.push(data_loader);

                                    result
                                }

                                IO::GraphQL { req_template, field_name, batch, .. } => {
                                    let graphql_data_loader =
                                        GraphqlDataLoader::new(runtime.clone(), *batch)
                                            .to_data_loader(
                                                upstream_batch.clone().unwrap_or_default(),
                                            );

                                    let result = Some(Expression::IO(IO::GraphQL {
                                        req_template: req_template.clone(),
                                        field_name: field_name.clone(),
                                        batch: *batch,
                                        dl_id: Some(DataLoaderId(gql_data_loaders.len())),
                                    }));

                                    gql_data_loaders.push(graphql_data_loader);

                                    result
                                }

                                IO::Grpc { req_template, group_by, .. } => {
                                    let data_loader = GrpcDataLoader {
                                        runtime: runtime.clone(),
                                        operation: req_template.operation.clone(),
                                        group_by: group_by.clone(),
                                    };
                                    let data_loader = data_loader
                                        .to_data_loader(upstream_batch.clone().unwrap_or_default());

                                    let result = Some(Expression::IO(IO::Grpc {
                                        req_template: req_template.clone(),
                                        group_by: group_by.clone(),
                                        dl_id: Some(DataLoaderId(grpc_data_loaders.len())),
                                    }));

                                    grpc_data_loaders.push(data_loader);

                                    result
                                }
                            },
                            _ => None,
                        })
                    });
                }
            }
        }

        let schema = blueprint
            .to_schema_with(SchemaModifiers::default().extensions(runtime.extensions.clone()));
        let auth = blueprint.server.auth.clone();
        let auth_ctx = GlobalAuthContext::new(auth);

        AppContext {
            schema,
            runtime,
            blueprint: SharedTrc::new(blueprint),
            http_data_loaders: SharedTrc::new(http_data_loaders),
            gql_data_loaders: SharedTrc::new(gql_data_loaders),
            grpc_data_loaders: SharedTrc::new(grpc_data_loaders),
            endpoints,
            auth_ctx: SharedTrc::new(auth_ctx),
        }
    }

    pub async fn execute(&self, request: impl Into<DynamicRequest>) -> Response {
        self.schema.execute(request).await
    }
}
