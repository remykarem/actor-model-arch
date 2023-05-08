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
        Self { audio_player }
    }
}

impl Handler<Sentence> for TtsActor {
    type Result = ResponseActFuture<Self, Result<(), ()>>;

    fn handle(&mut self, msg: Sentence, _: &mut Context<Self>) -> Self::Result {
        println!("Actor 2: Received {}", msg.0);
        let text = msg.0;
        Box::pin(
            // Some async computation
            async move {
                let client = Client::new();
                let api_url = "https://api.elevenlabs.io/v1/text-to-speech/";
                let voice_id = "EXAVITQu4vr4xnSDxMaL";
                let api_key =
                    env::var("ELEVENLABS_API_KEY").expect("ELEVENLABS_API_KEY must be set");

                let voice_settings = VoiceSettings {
                    stability: 0,
                    similarity_boost: 0,
                };
                let request_body = TextToSpeechRequest {
                    text: &text,
                    voice_settings,
                };
                let response = client
                    .post(&format!("{}{}", api_url, voice_id))
                    .header("xi-api-key", api_key)
                    .json(&request_body)
                    .send()
                    .await
                    .unwrap();

                response.bytes().await.unwrap().to_vec()
            }
            .into_actor(self) // converts future to ActorFuture
            .map(|res, actor, _ctx| {
                println!("Actor 2: Sending audio data with size {}", res.len());
                // Do some computation with actor's state or context
                actor.audio_player.do_send(Audio(res));
                Ok(())
            }),
        )
    }
}
