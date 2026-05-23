pub mod query;
pub mod types;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};

use query::Query;

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;