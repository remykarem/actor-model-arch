pub mod audio_player;
pub mod token_processor;
pub mod tts;

use std::time::Duration;

use actix::prelude::*;
use audio_player::AudioPlayerActor;
use token_processor::{Token, TokenProcessorActor};
use tts::TtsActor;

#[actix_rt::main]
async fn main() {
    let audio_player = SyncArbiter::start(1, || AudioPlayerActor {});
    let tts = TtsActor::with(audio_player).start();
    let token_proc = TokenProcessorActor::with(tts).start();

    for ch in "When you normally run Actors, there are multiple Actors running on the System's Arbiter thread, using its event loop. i'm fine, thank you.".chars() {
        actix_rt::time::sleep(Duration::from_millis(200)).await;
        token_proc.do_send(Token(ch));
    }

    // // stop system and exit
    System::current().stop();
}
