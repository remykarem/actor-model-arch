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
    // Initialise actors
    let audio_player = SyncArbiter::start(1, AudioPlayerActor::default);
    let tts = TtsActor::with(audio_player).start();
    let token_proc = TokenProcessorActor::with(tts).start();

    // Start the system
    for ch in "So, the actor model framework, right. Let's talk about that now. When you normally run Actors, there are multiple Actors running on the System's Arbiter thread, using its event loop. You know what I'm saying? Spring Cloud Config Bus is a feature of Spring Cloud Config that provides a mechanism for centralized configuration management and distribution in a distributed system, typically based on microservices architecture.".chars() {
        // Temp fix
        actix_rt::time::sleep(Duration::from_millis(10)).await;
        // Fire and forget
        token_proc.do_send(Token(ch));
    }

    // Send a message to tts (and other actors) to see if they are still processing anything
    // Put to 20s for now
    actix_rt::time::sleep(Duration::from_secs(20)).await;

    // // stop system and exit
    System::current().stop();
}
