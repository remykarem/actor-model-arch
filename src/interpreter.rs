use actix::prelude::*;
use serde::Deserialize;

use crate::{
    code_writer::{Code, CodeWriter},
    llm::{ChatMessage, LlmActor},
    tts_polly::{Sentence, TtsPollyActor},
    vectordb_qdrant::{QdrantStore, SearchRequest}, stt::{SttAction, Stt},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    Search(String),
    Writetofile { filename: String, content: String },
}

#[derive(Debug, Deserialize)]
struct ThoughtActions {
    thought: String,
    action: Option<Action>,
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
    qdrant: Addr<QdrantStore>,
    llm: Addr<LlmActor>,
    stt: Addr<Stt>,
    tts: Addr<TtsPollyActor>,
}

impl Actor for Interpreter {
    type Context = Context<Self>;
}

impl Interpreter {
    pub fn with(
        code_writer: Addr<CodeWriter>,
        tts: Addr<TtsPollyActor>,
        llm: Addr<LlmActor>,
        stt: Addr<Stt>,
        qdrant: Addr<QdrantStore>,
    ) -> Self {
        Self {
            code_writer,
            tts,
            llm,
            stt,
            qdrant,
        }
    }
}

impl Handler<Text> for Interpreter {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: Text, _ctx: &mut Self::Context) -> Self::Result {
        let thought_actions: ThoughtActions =
            serde_json::from_str(&msg.0).expect("Unabble to parse");
        let qdrant = self.qdrant.clone();
        let code_writer = self.code_writer.clone();
        let stt = self.stt.clone();
        let llm = self.llm.clone();
        let tts = self.tts.clone();

        Box::pin(async move {
            tts.do_send(Sentence(thought_actions.thought));

            // If there are actions, execute them
            if let Some(action) = thought_actions.action {
                let resp = match action {
                    Action::Writetofile { filename, content } => {
                        let _ = code_writer.send(Code { filename, content }).await.unwrap();
                        "dd".into()
                    }
                    Action::Search(stuff) => {
                        qdrant
                            .send(SearchRequest {
                                collection_name: "test_collection".into(),
                                vector: vec![0.05, 0.61, 0.76, 0.74],
                            })
                            .await
                            .unwrap()
                            .unwrap()
                    }
                };
                let _ = llm.send(ChatMessage(resp)).await;
            } else {
                let _ = stt.send(SttAction::RecordUntilSilence).await.unwrap();
            }

            Ok(())
        })
    }
}
