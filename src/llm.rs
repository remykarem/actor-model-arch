use std::sync::Arc;

use actix::{Actor, Addr, Context, Handler, Message, ResponseFuture};
use async_openai::types::{
    ChatCompletionRequestMessage,
    ChatCompletionResponseStreamMessage, CreateChatCompletionRequestArgs, Role,
};
use futures::StreamExt;
use serde::Deserialize;

use crate::token_processor::{Token, TokenProcessorActor};

pub const INITIAL_PROMPT: &str = include_str!("initial_prompt.md");

#[derive(Debug, Deserialize)]
pub struct ChatChoiceDelta {
    pub index: u32,
    pub delta: ChatCompletionResponseStreamMessage,
    pub finish_reason: Option<String>,
}

pub struct LlmActor {
    token_proc: Addr<TokenProcessorActor>,
    client: Arc<async_openai::Client>,
    messages: Vec<ChatCompletionRequestMessage>,
}

pub struct ChatMessage(pub String, pub Role);
pub struct ChatMessageFromAssistant(pub String);

impl Actor for LlmActor {
    type Context = Context<Self>;
}

impl Message for ChatMessage {
    type Result = Result<(), ()>;
}

impl Message for ChatMessageFromAssistant {
    type Result = Result<(), ()>;
}

impl Handler<ChatMessageFromAssistant> for LlmActor {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: ChatMessageFromAssistant, _ctx: &mut Self::Context) -> Self::Result {
        let message = ChatCompletionRequestMessage {
            content: msg.0,
            role: Role::Assistant,
            name: None,
        };

        self.messages.push(message);

        Ok(())
    }
}

impl Handler<ChatMessage> for LlmActor {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: ChatMessage, _ctx: &mut Self::Context) -> Self::Result {
        println!("LLM         : Received {:?}", msg.0);

        let message = ChatCompletionRequestMessage {
            content: msg.0,
            role: msg.1,
            name: None,
        };

        self.messages.push(message);
        let messages = self.messages.clone();

        let token_proc = self.token_proc.clone();
        let client = self.client.clone();

        Box::pin(async move {
            // Set up the request
            let request = CreateChatCompletionRequestArgs::default()
                .model("gpt-4")
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
        Self {
            token_proc,
            client: Arc::new(client),
            messages: vec![],
        }
    }
}
