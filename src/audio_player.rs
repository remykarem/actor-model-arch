use std::io::Cursor;

use actix::prelude::*;
use rodio::{Decoder, OutputStream, Sink};

pub struct Audio(pub Vec<u8>);

impl Message for Audio {
    type Result = Result<(), std::io::Error>;
}

pub struct AudioPlayerActor {
    // stream_handle: OutputStreamHandle,
}

impl AudioPlayerActor {
    pub async fn default() -> Self {
        // let sink = actix::spawn(async {
        // Open the default audio output device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.pause();
        //     sink
        // })
        // .await
        // .unwrap();

        Self {
            // stream_handle
        }
    }
}

impl Actor for AudioPlayerActor {
    type Context = SyncContext<Self>;
}

impl Handler<Audio> for AudioPlayerActor {
    type Result = Result<(), std::io::Error>;

    fn handle(&mut self, msg: Audio, _ctx: &mut SyncContext<Self>) -> Self::Result {
        println!("Actor 3: Received audio data {:?}", msg.0);

        // assume you have your audio data in a Vec<u8> called `audio_data`
        let cursor = Cursor::new(msg.0);
        let source = Decoder::new(cursor).unwrap();

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(source);
        sink.play();
        sink.sleep_until_end();

        Ok(())
    }
}
