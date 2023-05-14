pub mod audio_player;
pub mod code_writer;
pub mod interpreter;
pub mod llm;
pub mod stt;
pub mod token_processor;
pub mod tts;
pub mod tts_polly;
pub mod vectordb_qdrant;

use std::time::Duration;

use actix::prelude::*;
use audio_player::{AudioPlayerActor, Status, StatusRequest};
use code_writer::CodeWriter;
use interpreter::{GetObservations, Interpreter, Text, ThoughtActions};
use llm::{ChatMessage, LlmActor};
use stt::{Stt, SttAction};
use token_processor::TokenProcessorActor;
use tts_polly::{Sentence, TtsPollyActor};
use vectordb_qdrant::QdrantStore;

#[actix_rt::main]
async fn main() {
    // Tools
    let qdrant_client = QdrantStore::new().await.start();
    let code_writer = CodeWriter.start();

    // Interpreter
    let interpreter = Interpreter::with(code_writer, qdrant_client).start();

    // Initialise actors
    let audio_player = SyncArbiter::start(1, AudioPlayerActor::default);
    let tts = TtsPollyActor::with(audio_player.clone()).await.start();
    let token_proc = TokenProcessorActor::with(tts.clone()).start();

    // LLM
    let llm = LlmActor::with(token_proc.clone()).start();
    let llm_clone = llm.clone();

    let stt = SyncArbiter::start(1, move || {
        Stt::new(
            "/Users/raimibinkarim/Desktop/ggml-tiny.en.bin".to_string(),
            llm_clone.to_owned(),
        )
    });

    // Get the ball rolling
    // let _ = stt.send(SttAction::RecordUntilSilence).await.unwrap();

    actix_rt::time::sleep(Duration::from_secs(1)).await;

    // tts.do_send(Sentence("Hey there, I'm gonna write this file for you".into()));
    let _ = interpreter.send(Text(
        r#"
        {
            "actions": [
                {
                    "writetofile": {
                        "filename": "test.py",
                        "content": "import sys\n\narg = sys.argv[1]"
                    }
                },
                {
                    "writetofile": {
                        "filename": "testing.py",
                        "content": "import sys\n\narg = sys.argv[1]"
                    }
                }
            ]
        }"#
        .into(),
    )).await.unwrap();

    // Start the turn-based conversation
    loop {
        // Playing safe first
        actix_rt::time::sleep(Duration::from_secs(2)).await;

        // Once the system is stabilised
        // It is my turn when all the audio player is idle.
        let audio_status_request = audio_player.send(StatusRequest).await.unwrap().unwrap();
        let tts_status_request = tts.send(StatusRequest).await.unwrap().unwrap();
        let token_proc_status_request = token_proc.send(StatusRequest).await.unwrap().unwrap();
        let interpreter_status_request = interpreter.send(StatusRequest).await.unwrap().unwrap();
        if audio_status_request == Status::Idle
            && token_proc_status_request == Status::Idle
            && tts_status_request == Status::Idle
            && interpreter_status_request == Status::Idle
        {
            // Check if anything from interpreter
            let observation = interpreter.send(GetObservations).await.unwrap();

            // Continue with what its doing, or ask for input
            if let Some(observation) = observation {
                println!("--- observation: {} ---", observation);
                let _ = llm.send(ChatMessage(observation)).await.unwrap();
            } else {
                // Start recording
                let _ = stt.send(SttAction::RecordUntilSilence).await.unwrap();
            }
        };
    }

    // Stop system and exit
    // Currently unreachable code
    // System::current().stop();
}
