use std::sync::Arc;

use actix::prelude::*;
use serde::Deserialize;
use tokio::sync::Mutex;
use anyhow::Result;
use crate::{
    audio_player::{Status, StatusRequest},
    code_writer::{Code, CodeWriter},
    vectordb_qdrant::{QdrantStore, SearchRequest},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Action {
    Search { query: String, collection: String },
    Writetofile { filename: String, content: String },
    RetrieveDocuments,
    IndexDocuments,
    GetStdInput { prompt: String },
}

enum Ting {
    Documents(Vec<String>),
    Input(String),
}

#[derive(Debug, Deserialize)]
pub struct ThoughtActions {
    actions: Vec<Action>,
}

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct Text(pub String);

pub struct Interpreter {
    // code_writer: Addr<CodeWriter>,
    // qdrant: Addr<QdrantStore>,
    observations: Arc<Mutex<Vec<String>>>,
    memory: Arc<Mutex<Vec<Ting>>>,
    idle: bool,
}

impl Actor for Interpreter {
    type Context = Context<Self>;
}

impl Interpreter {
    pub fn with(
        // code_writer: Addr<CodeWriter>,
        // qdrant: Addr<QdrantStore>,
    ) -> Self {
        Self {
            // code_writer,
            // qdrant,
            observations: Arc::new(Mutex::new(vec![])),
            memory: Arc::new(Mutex::new(vec![])),
            idle: true,
        }
    }
}

impl Handler<Text> for Interpreter {
    type Result = ResponseFuture<Result<()>>;

    fn handle(&mut self, msg: Text, _ctx: &mut Self::Context) -> Self::Result {
        
        let thought_actions: ThoughtActions =
        serde_json::from_str(&msg.0).expect("Unable to parse");

        println!("Interpreter : Received {:?}", thought_actions.actions);

        // let qdrant = self.qdrant.clone();
        // let code_writer = self.code_writer.clone();

        self.idle = false;

        let memory = self.memory.clone();
        let observations = self.observations.clone();

        let b = Box::pin(
            async move {

                // If there are actions, execute them
                for action in thought_actions.actions {

                    match action {
                        // Action::Writetofile { filename, content } => {
                        //     println!("Interpreter : Sending to write to file");
                        //     let _ = code_writer.send(Code { filename, content }).await.unwrap();
                        //     observations.lock().await.push("value".into());
                        // }
                        Action::Search { .. } => {
                        //     qdrant
                        //     .send(SearchRequest {
                        //         collection_name: "test_collection".into(),
                        //         vector: vec![0.05, 0.61, 0.76, 0.74],
                        //     })
                        //     .await
                        //     .unwrap()
                        //     .unwrap();
                        //     observations.lock().await.push("value".into());
                        }
                        Action::RetrieveDocuments {} => {
                            // Get from memory
                            let docs = if let Ting::Input(yo) = memory.lock().await.pop().unwrap() {
                                // Retrieve documents
                                vec!["hello".into()]
                            } else {
                                panic!("Where's the stdinput...");
                            };

                            // Then add documents to memory
                            println!("Interpreter : Pushing {:?} to memory", docs);
                            memory.lock().await.push(Ting::Documents(docs));
                        }
                        Action::IndexDocuments {} => {
                            // Get from memory
                            if let Ting::Documents(docs) = memory.lock().await.pop().unwrap() {
                                // Then index docs
                                println!("Indexing docs {:?}", docs);
                            } else {
                                panic!("Where's the documents...");
                            };
                        }
                        Action::GetStdInput { prompt } => {
                            // Prompt
                            println!("Interpreter : {}", prompt);
                            
                            // Get stdinput
                            let mut input = String::new();
                            std::io::stdin().read_line(&mut input).unwrap();
                            println!("Interpreter : You entered {}", input);
                            
                            // Store in list
                            memory.lock().await.push(Ting::Input(input));
                        }
                        _ => {}
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
    type Result = Result<Status>;

    fn handle(&mut self, _msg: StatusRequest, _ctx: &mut Context<Self>) -> Self::Result {
        // println!("Interpreter : Received status request. Idle? {}", self.idle);

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
