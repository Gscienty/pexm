use std::sync::Arc;

use auth_lib::AuthClaim;
use axum::{extract::State, response::IntoResponse};

use crate::graphql::{query::{Query, Mutation}, AppSchema};
use crate::{AppJotProvider, AppState};

pub async fn graphql_handler(
    State(state): State<Arc<AppState>>,
    AuthClaim(claim, _): AuthClaim<AppJotProvider>,
    request: async_graphql_axum::GraphQLRequest,
) -> impl IntoResponse {
    let schema = AppSchema::build(
        Query,
        Mutation,
        async_graphql::EmptySubscription,
    )
    .data(claim)
    .data(state.dal.clone())
    .finish();

    async_graphql_axum::GraphQLResponse::from(schema.execute(request.into_inner()).await)
}