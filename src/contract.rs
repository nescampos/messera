#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use async_trait::async_trait;
use linera_sdk::{
    base::{ChannelName, Destination, SessionId, WithContractAbi},
    contract::system_api,
    ApplicationCallResult, CalleeContext, Contract, ExecutionResult, MessageContext,
    OperationContext, SessionCallResult, ViewStateStorage,
};
use linera_views::views::ViewError;
use messera::{Key, Message, Operation, MyMessage};
use state::Messera;
use thiserror::Error;

/// The channel name the application uses for cross-chain messages about new posts.
const MESSAGES_CHANNEL_NAME: &[u8] = b"messages";
const LAST_MESSAGES: usize = 20;


linera_sdk::contract!(Messera);

impl WithContractAbi for Messera {
    type Abi = messera::ApplicationAbi;
}

#[async_trait]
impl Contract for Messera {
    type Error = Error;
    type Storage = ViewStateStorage<Self>;

    async fn initialize(
        &mut self,
        _context: &OperationContext,
        _argument: (),
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        Ok(ExecutionResult::default())
    }

    async fn execute_operation(
        &mut self,
        _context: &OperationContext,
        operation: Operation,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        match operation {
            Operation::RequestSubscribe(chain_id) => {
                Ok(ExecutionResult::default().with_message(chain_id, Message::RequestSubscribe))
            }
            Operation::RequestUnsubscribe(chain_id) => {
                Ok(ExecutionResult::default().with_message(chain_id, Message::RequestUnsubscribe))
            }
            Operation::Content(text) => self.execute_content_operation(text).await,
        }
    }

    async fn execute_message(
        &mut self,
        context: &MessageContext,
        message: Message,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        let mut result = ExecutionResult::default();
        match message {
            Message::RequestSubscribe => result.subscribe.push((
                ChannelName::from(MESSAGES_CHANNEL_NAME.to_vec()),
                context.message_id.chain_id,
            )),
            Message::RequestUnsubscribe => result.unsubscribe.push((
                ChannelName::from(MESSAGES_CHANNEL_NAME.to_vec()),
                context.message_id.chain_id,
            )),
            Message::Messages { count, messages } => self.execute_received_message(context, count, messages)?,
        }
        Ok(result)
    }

    async fn handle_application_call(
        &mut self,
        _context: &CalleeContext,
        _call: (),
        _forwarded_sessions: Vec<SessionId>,
    ) -> Result<ApplicationCallResult<Self::Message, Self::Response, Self::SessionState>, Self::Error>
    {
        Err(Error::ApplicationCallsNotSupported)
    }

    async fn handle_session_call(
        &mut self,
        _context: &CalleeContext,
        _state: Self::SessionState,
        _call: (),
        _forwarded_sessions: Vec<SessionId>,
    ) -> Result<SessionCallResult<Self::Message, Self::Response, Self::SessionState>, Self::Error>
    {
        Err(Error::SessionsNotSupported)
    }
}

impl Messera {
    async fn execute_content_operation(
        &mut self,
        text: String,
    ) -> Result<ExecutionResult<Message>, Error> {
        let timestamp = system_api::current_system_time();
        self.my_messages.push(MyMessage { timestamp, text });
        let count = self.my_messages.count();
        let mut messages = vec![];
        for index in (0..count).rev().take(LAST_MESSAGES) {
            let maybe_message = self.my_messages.get(index).await?;
            let my_message = maybe_message
                .expect("message with valid index missing; this is a bug for messera!");
            messages.push(my_message);
        }
        let count = count as u64;
        let dest = Destination::Subscribers(ChannelName::from(MESSAGES_CHANNEL_NAME.to_vec()));
        Ok(ExecutionResult::default().with_message(dest, Message::Messages { count, messages }))
    }

    fn execute_received_message(
        &mut self,
        context: &MessageContext,
        count: u64,
        messages: Vec<MyMessage>,
    ) -> Result<(), Error> {
        for (index, message) in (0..count).rev().zip(messages) {
            let key = Key {
                timestamp: message.timestamp,
                author: context.message_id.chain_id,
                index,
            };
            self.received_messages.insert(&key, message.text)?;
        }
        Ok(())
    }
}

/// An error that can occur during the contract execution.
#[derive(Debug, Error)]
pub enum Error {
    /// Social application doesn't support any cross-application sessions.
    #[error("Messera doesn't support any cross-application sessions")]
    SessionsNotSupported,

    /// Social application doesn't support any cross-application sessions.
    #[error("Messera doesn't support any application calls")]
    ApplicationCallsNotSupported,

    /// View error.
    #[error(transparent)]
    View(#[from] ViewError),

    /// Failed to deserialize BCS bytes
    #[error("Failed to deserialize BCS bytes")]
    BcsError(#[from] bcs::Error),

    /// Failed to deserialize JSON string
    #[error("Failed to deserialize JSON string")]
    JsonError(#[from] serde_json::Error),
}
