use actix::prelude::*;
use serde::Deserialize;

use crate::{code_writer::{CodeWriter, Code}, tts_polly::{TtsPollyActor, Sentence}};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    Search(String),
    Writetofile { filename: String, content: String },
}

#[derive(Debug, Deserialize)]
struct ThoughtActions {
    thought: String,
    actions: Vec<Action>,
}

#[derive(Debug, Deserialize)]
struct Observation {
    observation: String,
}

pub struct Text(pub String);

impl Message for Text {
    type Result = Result<(), ()>;
}

pub struct Interpreter {
    code_writer: Addr<CodeWriter>,
    tts: Addr<TtsPollyActor>,
}

impl Actor for Interpreter {
    type Context = Context<Self>;
}

impl Interpreter {
    pub fn with(code_writer: Addr<CodeWriter>, tts: Addr<TtsPollyActor>) -> Self {
        Self { code_writer, tts }
    }
}

impl Handler<Text> for Interpreter {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: Text, _ctx: &mut Self::Context) -> Self::Result {
        let thought_actions: ThoughtActions =
            serde_json::from_str(&msg.0).expect("Unabble to parse");
        let code_writer = self.code_writer.clone();
        let tts = self.tts.clone();

        Box::pin(async move {

            tts.do_send(Sentence(thought_actions.thought));

            for action in thought_actions.actions {
                match action {
                    Action::Writetofile { filename, content } => {
                        let _ = code_writer.send(Code { filename, content}).await.unwrap();
                    },
                    _ => {}
                }
            }

            Ok(())
        })
    }
}
