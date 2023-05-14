use actix::prelude::*;
use aws_sdk_polly as polly;
use polly::{
    types::{Engine, OutputFormat},
    Client,
};

use crate::audio_player::{Audio, AudioPlayerActor, Status, StatusRequest};

pub struct Sentence(pub String);

impl Message for Sentence {
    type Result = Result<(), ()>;
}

pub struct TtsPollyActor {
    audio_player: Addr<AudioPlayerActor>,
    client: Client,
    idle: bool,
}

impl Actor for TtsPollyActor {
    type Context = Context<Self>;
}

impl TtsPollyActor {
    pub async fn with(audio_player: Addr<AudioPlayerActor>) -> Self {
        let config = aws_config::load_from_env().await;
        let client = polly::Client::new(&config);
        let idle = true;
        Self {
            audio_player,
            client,
            idle,
        }
    }
}

impl Handler<Sentence> for TtsPollyActor {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: Sentence, _ctx: &mut Self::Context) -> Self::Result {
        println!("TTS          : Received {}", msg.0.trim());

        let audio_player = self.audio_player.clone();
        let client = self.client.clone();

        self.idle = false;

        let b = Box::pin(async move {
            let resp = client
                .synthesize_speech()
                .engine(Engine::Standard)
                .voice_id(polly::types::VoiceId::Matthew)
                .output_format(OutputFormat::Mp3)
                .text(msg.0.trim())
                .send()
                .await
                .unwrap();

            let data = resp.audio_stream.collect().await.unwrap().to_vec();

            let _ = audio_player.send(Audio(data)).await.unwrap();

            Ok(())
        });

        self.idle = true;

        b
    }
}

impl Handler<StatusRequest> for TtsPollyActor {
    type Result = Result<Status, std::io::Error>;

    fn handle(&mut self, _msg: StatusRequest, _ctx: &mut Context<Self>) -> Self::Result {
        // println!("TTS         : Received status request");

        if self.idle {
            Ok(Status::Idle)
        } else {
            Ok(Status::Busy)
        }
    }
}
