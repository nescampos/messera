#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use async_graphql::{EmptySubscription, Object, Request, Response, Schema};
use self::state::Messera;
use async_trait::async_trait;
use linera_sdk::{
    base::{ChainId, WithServiceAbi},
    QueryContext, Service, ViewStateStorage,
};
use std::sync::Arc;
use thiserror::Error;
use linera_views::views::ViewError;
use messera::Operation;

linera_sdk::service!(Messera);

impl WithServiceAbi for Messera {
    type Abi = messera::ApplicationAbi;
}

#[async_trait]
impl Service for Messera {
    type Error = Error;
    type Storage = ViewStateStorage<Self>;

    async fn query_application(
        self: Arc<Self>,
        _context: &QueryContext,
        request: Request,
    ) -> Result<Response, Self::Error> {
        let schema = Schema::build(self.clone(), MutationRoot, EmptySubscription).finish();
        let response = schema.execute(request).await;
        Ok(response)
    }
}

struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn subscribe(&self, chain_id: ChainId) -> Vec<u8> {
        bcs::to_bytes(&Operation::RequestSubscribe(chain_id)).unwrap()
    }

    async fn unsubscribe(&self, chain_id: ChainId) -> Vec<u8> {
        bcs::to_bytes(&Operation::RequestUnsubscribe(chain_id)).unwrap()
    }

    async fn content(&self, text: String) -> Vec<u8> {
        bcs::to_bytes(&Operation::Content(text)).unwrap()
    }
}

/// An error that can occur during the service execution.
#[derive(Debug, Error)]
pub enum Error {
    /// Invalid query.
    #[error("Invalid query")]
    InvalidQuery(#[from] serde_json::Error),

    /// Serialization error.
    #[error(transparent)]
    Serialization(#[from] bcs::Error),

    /// View error.
    #[error(transparent)]
    View(#[from] ViewError),
}