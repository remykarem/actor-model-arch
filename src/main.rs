pub mod audio_player;
pub mod llm;
pub mod stt;
pub mod token_processor;
pub mod tts;
pub mod tts_polly;
pub mod code_writer;

use std::time::Duration;

use actix::prelude::*;
use audio_player::{AudioPlayerActor, Status, StatusRequest};
use llm::LlmActor;
use stt::{Stt, SttAction};
use token_processor::TokenProcessorActor;
use tts_polly::TtsPollyActor;

#[actix_rt::main]
async fn main() {
    // Initialise actors
    let audio_player = SyncArbiter::start(1, AudioPlayerActor::default);
    let tts = TtsPollyActor::with(audio_player.clone()).await.start();
    let token_proc = TokenProcessorActor::with(tts.clone()).start();
    let llm = LlmActor::with(token_proc.clone()).start();
    let stt = SyncArbiter::start(1, move || {
        Stt::new(
            "/Users/raimibinkarim/Desktop/ggml-tiny.en.bin".to_string(),
            llm.clone(),
        )
    });

    // Get the ball rolling
    let _ = stt.send(SttAction::RecordUntilSilence).await.unwrap();

    // Start the turn-based conversation
    loop {
        // Playing safe first
        actix_rt::time::sleep(Duration::from_secs(2)).await;

        // It is my turn when all the audio player is idle.
        let audio_status_request = audio_player.send(StatusRequest).await.unwrap().unwrap();
        let tts_status_request = tts.send(StatusRequest).await.unwrap().unwrap();
        let token_proc_status_request = token_proc.send(StatusRequest).await.unwrap().unwrap();
        if audio_status_request == Status::Idle
            && token_proc_status_request == Status::Idle
            && tts_status_request == Status::Idle
        {
            // Start recording
            let _ = stt.send(SttAction::RecordUntilSilence).await.unwrap();
        };
    }

    // Stop system and exit
    // Currently unreachable code
    // System::current().stop();
}
