use std::env;
use anyhow::Result;
use actix::prelude::*;
use reqwest::Client;
use serde::Serialize;

use crate::audio_player::{Audio, AudioPlayerActor, Status, StatusRequest};

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct Utterance(pub String);

pub struct TtsActor {
    client: Client,
    audio_player: Addr<AudioPlayerActor>,
    idle: bool,
}

impl Actor for TtsActor {
    type Context = Context<Self>;
}

#[derive(Serialize)]
struct VoiceSettings {
    stability: u32,
    similarity_boost: u32,
}

#[derive(Serialize)]
struct TextToSpeechRequest<'a> {
    text: &'a str,
    voice_settings: VoiceSettings,
}

impl TtsActor {
    pub fn with(audio_player: Addr<AudioPlayerActor>) -> Self {
        let client = Client::new();
        let idle = true;
        Self {
            audio_player,
            client,
            idle,
        }
    }
}

impl Handler<Utterance> for TtsActor {
    type Result = ResponseFuture<Result<()>>;

    fn handle(&mut self, msg: Utterance, _: &mut Context<Self>) -> Self::Result {
        println!("TTS         : Received {}", msg.0);

        let text = msg.0;
        let audio_player = self.audio_player.clone();
        let client = self.client.clone(); // Client is already ARC-wrapped

        self.idle = false;

        let b = Box::pin(async move {
            let api_url = "https://api.elevenlabs.io/v1/text-to-speech/";
            let voice_id = "EXAVITQu4vr4xnSDxMaL";
            let api_key = env::var("ELEVENLABS_API_KEY").expect("ELEVENLABS_API_KEY must be set");
            let url = format!("{}{}", api_url, voice_id);

            let voice_settings = VoiceSettings {
                stability: 0,
                similarity_boost: 0,
            };
            let request_body = TextToSpeechRequest {
                text: &text,
                voice_settings,
            };
            let response = client
                .post(&url)
                .header("xi-api-key", api_key)
                .json(&request_body)
                .send()
                .await
                .unwrap();

            let data = response.bytes().await.unwrap().to_vec();

            // Ensures that messages are in order
            let _ = audio_player.send(Audio(data)).await.unwrap();

            Ok(())
        });

        self.idle = true;

        b
    }
}

impl Handler<StatusRequest> for TtsActor {
    type Result = Result<Status>;

    fn handle(&mut self, _msg: StatusRequest, _ctx: &mut Context<Self>) -> Self::Result {
        println!("TTS         : Received status request");

        if self.idle {
            Ok(Status::Idle)
        } else {
            Ok(Status::Busy)
        }
    }
}
