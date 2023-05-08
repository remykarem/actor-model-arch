use actix::prelude::*;

use crate::{tts::{Sentence, TtsActor}, audio_player::{StatusRequest, Status}};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Token(pub String);

pub struct TokenProcessorActor {
    token_buffer: Vec<String>,
    tts: Addr<TtsActor>,
    idle: bool,
}

impl Actor for TokenProcessorActor {
    type Context = Context<Self>;
}

impl TokenProcessorActor {
    pub fn with(tts: Addr<TtsActor>) -> Self {
        Self {
            token_buffer: vec![],
            tts,
            idle: true,
        }
    }
}

impl Handler<Token> for TokenProcessorActor {
    type Result = ();

    fn handle(&mut self, msg: Token, _ctx: &mut Context<Self>) -> Self::Result {
        // println!("Actor 1: Received {}", msg.0);

        if msg.0.starts_with('?') | msg.0.starts_with('.') | msg.0.starts_with('!') {
            let sentence: String = self.token_buffer.drain(..).collect();
            // println!("Actor 1: Pushing {} to Actor 2", sentence);
            self.tts.do_send(Sentence(format!("{} {}", sentence, msg.0)));
            self.idle = true;
        } else {
            self.token_buffer.push(msg.0);
            self.idle = false;
        }
    }
}

impl Handler<StatusRequest> for TokenProcessorActor {
    type Result = Result<Status, std::io::Error>;

    fn handle(&mut self, _msg: StatusRequest, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Token Proc  : Received status request");

        if self.idle && self.token_buffer.is_empty() {
            Ok(Status::Idle)
        } else {
            Ok(Status::Busy)
        }
    }
}
