use actix::prelude::*;

use anyhow::Result;

use std::sync::Arc;

use qdrant_client::{
    prelude::*,
    qdrant::{with_payload_selector::SelectorOptions, PayloadIncludeSelector, WithPayloadSelector},
};

#[derive(Message)]
#[rtype(result = "Result<String>")]
pub struct SearchRequest {
    pub collection_name: String,
    pub vector: Vec<f32>,
}

pub struct QdrantStore {
    client: Arc<QdrantClient>,
}

impl Actor for QdrantStore {
    type Context = Context<Self>;
}

impl QdrantStore {
    pub async fn new() -> Self {
        let config = QdrantClientConfig::from_url("http://localhost:6334");
        let client = QdrantClient::new(Some(config))
            .await
            .expect("Failed to create client");

        Self {
            client: Arc::new(client),
        }
    }
}

impl Handler<SearchRequest> for QdrantStore {
    type Result = ResponseFuture<Result<String>>;

    fn handle(&mut self, msg: SearchRequest, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();

        Box::pin(async move {
            let search_result = client
                .search_points(&SearchPoints {
                    collection_name: msg.collection_name,
                    vector: msg.vector,
                    filter: None,
                    limit: 1,
                    with_vectors: None,
                    with_payload: Some(WithPayloadSelector {
                        selector_options: Some(SelectorOptions::Include(PayloadIncludeSelector {
                            fields: vec!["city".to_string()],
                        })),
                    }),
                    params: None,
                    score_threshold: None,
                    offset: None,
                    ..Default::default()
                })
                .await
                .expect("Error searching");

            let point = search_result.result.first().unwrap();

            let m = point.payload.get(&"city".to_string()).unwrap();
            let v = match &m.kind {
                Some(v) => match v {
                    qdrant_client::qdrant::value::Kind::StringValue(value) => value.clone(),
                    _ => todo!(),
                },
                None => todo!(),
            };

            // vec!["ddd".into()];

            Ok(v)
        })
    }
}
