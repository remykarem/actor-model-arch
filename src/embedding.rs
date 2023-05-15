use actix::prelude::*;
use rust_bert::pipelines::sentence_embeddings::{SentenceEmbeddingsModel, SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType::AllMiniLmL6V2, Embedding};

#[derive(Message)]
#[rtype(result = "Embedding")]
pub struct EmbeddingQuery(pub String);

impl AsRef<str> for EmbeddingQuery {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

pub struct EmbeddingModel {
    model: SentenceEmbeddingsModel,
}

impl Default for EmbeddingModel {
    fn default() -> Self {
        println!("Embedding    : Creating model");
        let model = SentenceEmbeddingsBuilder::remote(AllMiniLmL6V2)
            .create_model()
            .expect("Cannot create model");
        println!("Embedding    : Model created");
        Self { model }
    }
}

impl Actor for EmbeddingModel {
    type Context = SyncContext<Self>;
}

impl Handler<EmbeddingQuery> for EmbeddingModel {
    type Result = Embedding;

    fn handle(&mut self, embedding_query: EmbeddingQuery, _ctx: &mut SyncContext<Self>) -> Self::Result {
        println!("Embedding    : Received embedding query");

        self.model.encode(&[embedding_query]).expect("Cannot embed query").pop().expect("Empty result")
    }
}
