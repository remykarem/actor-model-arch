use actix::prelude::*;
use tokio::{fs::File, io::AsyncWriteExt};
use anyhow::Result;

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct Code {
    pub filename: String,
    pub content: String,
}

pub struct CodeWriter;

impl Actor for CodeWriter {
    type Context = Context<Self>;
}

impl Handler<Code> for CodeWriter {
    type Result = ResponseFuture<Result<()>>;

    fn handle(&mut self, msg: Code, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(async move {
            let mut file = File::create(msg.filename).await.unwrap();
            file.write_all(msg.content.as_bytes()).await.unwrap();
            Ok(())
        })
    }
}
