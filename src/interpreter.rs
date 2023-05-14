use actix::prelude::*;
use serde::Deserialize;

use crate::{
    code_writer::{Code, CodeWriter},
    vectordb_qdrant::{QdrantStore, SearchRequest}, audio_player::{StatusRequest, Status},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Search(String),
    Writetofile { filename: String, content: String },
}

#[derive(Debug, Deserialize)]
pub struct ThoughtActions {
    thought: String,
    action: Option<Action>,
}

pub struct Text(pub String);

impl Message for Text {
    type Result = Result<String, ()>;
}

pub struct Interpreter {
    code_writer: Addr<CodeWriter>,
    qdrant: Addr<QdrantStore>,
    observation: Option<String>,
    idle: bool,
}

impl Actor for Interpreter {
    type Context = Context<Self>;
}

impl Interpreter {
    pub fn with(code_writer: Addr<CodeWriter>, qdrant: Addr<QdrantStore>) -> Self {
        Self {
            code_writer,
            qdrant,
            observation: None,
            idle: true,
        }
    }
}

impl Handler<Text> for Interpreter {
    type Result = ResponseActFuture<Self, Result<String, ()>>;

    fn handle(&mut self, msg: Text, _ctx: &mut Self::Context) -> Self::Result {
        println!("Interpreter : Received Text request");

        let thought_actions: ThoughtActions =
            serde_json::from_str(&msg.0).expect("Unable to parse");
        let qdrant = self.qdrant.clone();
        let code_writer = self.code_writer.clone();

        self.idle = false;

        Box::pin(
            async move {
                println!("Interpreter : {:?}", thought_actions.action);

                // If there are actions, execute them
                if let Some(action) = thought_actions.action {
                    match action {
                        Action::Writetofile { filename, content } => {
                            println!("Interpreter : Sending to write to file");
                            let _ = code_writer.send(Code { filename, content }).await.unwrap();
                            "dd".into()
                        }
                        Action::Search(stuff) => qdrant
                            .send(SearchRequest {
                                collection_name: "test_collection".into(),
                                vector: vec![0.05, 0.61, 0.76, 0.74],
                            })
                            .await
                            .unwrap()
                            .unwrap(),
                    }
                } else {
                    "d".into()
                }
            }
            .into_actor(self)
            .map(|res, act, _ctx| {
                act.observation = Some(res);
                act.idle = true;
                Ok("res".into())
            })
        )
    }
}

impl Handler<StatusRequest> for Interpreter {
    type Result = Result<Status, std::io::Error>;

    fn handle(&mut self, _msg: StatusRequest, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Interpreter : Received status request");

        if self.idle{
            Ok(Status::Idle)
        } else {
            Ok(Status::Busy)
        }
    }
}

pub struct GetObservations;

impl Message for GetObservations {
    type Result = Option<String>;
}

impl Handler<GetObservations> for Interpreter {
    type Result = Option<String>;

    fn handle(&mut self, _msg: GetObservations, _ctx: &mut Context<Self>) -> Self::Result {
        self.observation.clone()
    }
}
