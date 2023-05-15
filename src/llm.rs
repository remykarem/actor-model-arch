use std::sync::Arc;

use actix::{Actor, Addr, Context, Handler, Message, ResponseActFuture, WrapFuture, ActorFutureExt};
use async_openai::types::{
    ChatCompletionRequestMessage,
    ChatCompletionResponseStreamMessage, CreateChatCompletionRequestArgs, Role,
};
use futures::StreamExt;
use serde::Deserialize;

use crate::{token_processor::{Token, TokenProcessorActor}, audio_player::{StatusRequest, Status}};

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
    idle: bool,
}

#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct ChatMessage(pub String, pub Role);

impl Actor for LlmActor {
    type Context = Context<Self>;
}

impl Handler<ChatMessage> for LlmActor {
    type Result = ResponseActFuture<Self, Result<(), ()>>;

    fn handle(&mut self, msg: ChatMessage, _ctx: &mut Self::Context) -> Self::Result {
        println!("LLM         : Received {:?}", msg.0);

        // Prepare the request
        let message = ChatCompletionRequestMessage {
            content: msg.0,
            role: msg.1,
            name: None,
        };

        // Update state
        self.idle = false;
        self.messages.push(message);

        // Clone the actors for the async task
        let token_proc = self.token_proc.clone();
        let client = self.client.clone();
        let messages = self.messages.clone();

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
            
            // Create buffer to store the reply
            let mut content: Vec<String> = Vec::with_capacity(1024);

            // Process the stream
            while let Some(result) = response.next().await {
                let mut response = result.expect("Error while reading response");
                let something = response.choices.pop().unwrap();

                if let Some(token) = something.delta.content {
                    // Send to the processor
                    token_proc.send(Token(token.clone())).await.unwrap();

                    // Update the reply buffer
                    content.push(token);
                }
            }

            content.into_iter().collect()
        }.into_actor(self).map(|content, act, _ctx| {
            // Update the states
            act.messages.push(ChatCompletionRequestMessage { role: Role::Assistant, content, name: None });
            act.idle = true;
            Ok(())
        }))
    }
}

impl LlmActor {
    pub fn with(token_proc: Addr<TokenProcessorActor>) -> Self {
        let client = async_openai::Client::new();
        Self {
            token_proc,
            client: Arc::new(client),
            messages: vec![],
            idle: true,
        }
    }
}

impl Handler<StatusRequest> for LlmActor {
    type Result = Result<Status, std::io::Error>;

    fn handle(&mut self, _msg: StatusRequest, _ctx: &mut Context<Self>) -> Self::Result {
        if self.idle {
            Ok(Status::Idle)
        } else {
            Ok(Status::Busy)
        }
    }
}
