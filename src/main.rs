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
    let audio_player = SyncArbiter::start(1, AudioPlayerActor::default);
    let tts = TtsActor::with(audio_player).start();
    let token_proc = TokenProcessorActor::with(tts).start();

    for ch in "So, the actor model framework, right. Let's talk about that now. When you normally run Actors, there are multiple Actors running on the System's Arbiter thread, using its event loop. You know what I'm saying? Spring Cloud Config Bus is a feature of Spring Cloud Config that provides a mechanism for centralized configuration management and distribution in a distributed system, typically based on microservices architecture.".chars() {
        actix_rt::time::sleep(Duration::from_millis(50)).await;
        token_proc.do_send(Token(ch));
    }

    actix_rt::time::sleep(Duration::from_secs(20)).await;

    // // stop system and exit
    System::current().stop();
}
