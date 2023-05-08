use std::env;

use actix::prelude::*;
use reqwest::Client;
use serde::Serialize;

use crate::audio_player::{Audio, AudioPlayerActor};

pub struct Sentence(pub String);

impl Message for Sentence {
    type Result = Result<(), ()>;
}

pub struct TtsActor {
    client: Client,
    audio_player: Addr<AudioPlayerActor>,
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
        Self { audio_player, client }
    }
}

impl Handler<Sentence> for TtsActor {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: Sentence, _: &mut Context<Self>) -> Self::Result {
        println!("Actor 2: Received {}", msg.0);

        let text = msg.0;
        let audio_player = self.audio_player.clone();
        let client = self.client.clone();  // Client is already ARC-wrapped

        Box::pin(
            async move {
                let api_url = "https://api.elevenlabs.io/v1/text-to-speech/";
                let voice_id = "EXAVITQu4vr4xnSDxMaL";
                let api_key =
                    env::var("ELEVENLABS_API_KEY").expect("ELEVENLABS_API_KEY must be set");
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
            }
        )

    }
}
