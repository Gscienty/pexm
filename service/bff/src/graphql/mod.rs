pub mod query;
pub mod types;

use async_graphql::{EmptySubscription, Schema};

use query::{Query, Mutation};

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;