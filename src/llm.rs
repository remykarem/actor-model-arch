use std::sync::Arc;

use actix::{Actor, Addr, Context, Handler, Message, ResponseFuture};
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionResponseStreamMessage,
    CreateChatCompletionRequestArgs, Role,
};
use futures::StreamExt;
use serde::Deserialize;

use crate::token_processor::{TokenProcessorActor, Token};

pub const INITIAL_PROMPT: &str = include_str!("prompt.md");

#[derive(Debug, Deserialize, Clone)]
pub struct ChatChoiceDelta {
    pub index: u32,
    pub delta: ChatCompletionResponseStreamMessage,
    pub finish_reason: Option<String>,
}

pub struct LlmActor {
    token_proc: Addr<TokenProcessorActor>,
    client: Arc<async_openai::Client>,
}

pub struct ChatMessage(pub String);

impl Actor for LlmActor {
    type Context = Context<Self>;
}

impl Message for ChatMessage {
    type Result = Result<(), ()>;
}

impl Handler<ChatMessage> for LlmActor {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: ChatMessage, _ctx: &mut Self::Context) -> Self::Result {
        let messages = vec![ChatCompletionRequestMessage {
            role: Role::Assistant,
            content: msg.0,
            name: None,
        }];
        let token_proc = self.token_proc.clone();
        let client = self.client.clone();

        Box::pin(async move {
            // Set up the request
            let request = CreateChatCompletionRequestArgs::default()
                .model("gpt-3.5-turbo")
                .messages(messages)
                .stream(true)
                .build()
                .unwrap();

            // Make the request
            let mut response = client.chat().create_stream(request).await.unwrap();

            // Process the stream
            while let Some(result) = response.next().await {
                let mut response = result.expect("Error while reading response");
                let something = response.choices.pop().unwrap();

                if let Some(token) = something.delta.content {
                    token_proc.send(Token(token)).await.unwrap();
                }
            }

            Ok(())
        })
    }
}

impl LlmActor {
    pub fn with(token_proc: Addr<TokenProcessorActor>) -> Self {
        let client = async_openai::Client::new();
        Self { token_proc, client: Arc::new(client) }
    }
}
