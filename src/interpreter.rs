use std::sync::Arc;

use actix::prelude::*;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{
    audio_player::{Status, StatusRequest},
    code_writer::{Code, CodeWriter},
    vectordb_qdrant::{QdrantStore, SearchRequest},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Search(String),
    Writetofile { filename: String, content: String },
}

#[derive(Debug, Deserialize)]
pub struct ThoughtActions {
    actions: Vec<Action>,
}

pub struct Text(pub String);

impl Message for Text {
    type Result = Result<(), ()>;
}

pub struct Interpreter {
    code_writer: Addr<CodeWriter>,
    qdrant: Addr<QdrantStore>,
    observations: Arc<Mutex<Vec<String>>>,
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
            observations: Arc::new(Mutex::new(vec![])),
            idle: true,
        }
    }
}

impl Handler<Text> for Interpreter {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: Text, _ctx: &mut Self::Context) -> Self::Result {
        
        let thought_actions: ThoughtActions =
        serde_json::from_str(&msg.0).expect("Unable to parse");

        println!("Interpreter : Received {:?}", thought_actions.actions);

        let qdrant = self.qdrant.clone();
        let code_writer = self.code_writer.clone();

        self.idle = false;

        let observations = self.observations.clone();

        let b = Box::pin(
            async move {

                // If there are actions, execute them
                for action in thought_actions.actions {
                    match action {
                        Action::Writetofile { filename, content } => {
                            println!("Interpreter : Sending to write to file");
                            let _ = code_writer.send(Code { filename, content }).await.unwrap();
                            observations.lock().await.push("value".into());
                        }
                        Action::Search(stuff) => {
                            qdrant
                            .send(SearchRequest {
                                collection_name: "test_collection".into(),
                                vector: vec![0.05, 0.61, 0.76, 0.74],
                            })
                            .await
                            .unwrap()
                            .unwrap();
                            observations.lock().await.push("value".into());
                        }
                    }
                }
                Ok(())
            }
        );

        self.idle = true;

        b
    }
}

impl Handler<StatusRequest> for Interpreter {
    type Result = Result<Status, std::io::Error>;

    fn handle(&mut self, _msg: StatusRequest, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Interpreter : Received status request");

        if self.idle {
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
    type Result = ResponseFuture<Option<String>>;

    fn handle(&mut self, _msg: GetObservations, _ctx: &mut Context<Self>) -> Self::Result {
        let observations = self.observations.clone();
        Box::pin(
            async move {
                let mut m = observations.lock().await;
                m.pop()
            }
        )
    }
}
