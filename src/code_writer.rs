use std::sync::Arc;

use actix::prelude::*;
use tokio::{fs::File, io::AsyncWriteExt, sync::Mutex};

pub struct Code(pub String);

impl Message for Code {
    type Result = Result<(), ()>;
}

pub struct CodeWriter {
    file: Arc<Mutex<File>>,
}

impl Actor for CodeWriter {
    type Context = Context<Self>;
}

impl CodeWriter {
    pub async fn new() -> Self {
        Self {
            file: Arc::new(Mutex::new(File::create("test.rs").await.unwrap())),
        }
    }
}

impl Handler<Code> for CodeWriter {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: Code, _ctx: &mut Self::Context) -> Self::Result {
        let file = self.file.clone();
        Box::pin(async move {
            file.try_lock()
                .unwrap()
                .try_clone()
                .await
                .unwrap()
                .write_all(msg.0.as_bytes())
                .await
                .unwrap();
            Ok(())
        })
    }
}
