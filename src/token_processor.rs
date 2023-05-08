use actix::prelude::*;

use crate::tts::{Sentence, TtsActor};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Token(pub char);

pub struct TokenProcessorActor {
    token_buffer: Vec<char>,
    tts: Addr<TtsActor>,
}

impl Actor for TokenProcessorActor {
    type Context = Context<Self>;
}

impl TokenProcessorActor {
    pub fn with(tts: Addr<TtsActor>) -> Self {
        Self {
            token_buffer: vec![],
            tts,
        }
    }
}

impl Handler<Token> for TokenProcessorActor {
    type Result = ();

    fn handle(&mut self, msg: Token, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Actor 1: Received {}", msg.0);
        self.token_buffer.push(msg.0);
        
        if msg.0 == '.' {
            let sentence: String = self.token_buffer.drain(..).collect();
            println!("Actor 1: Pushing {} to Actor 2", sentence);
            self.tts.do_send(Sentence(sentence));
        }
    }
}
