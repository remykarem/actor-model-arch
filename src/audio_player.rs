use std::io::Cursor;

use actix::prelude::*;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

pub struct Audio(pub Vec<u8>);

impl Message for Audio {
    type Result = Result<(), std::io::Error>;
}

pub struct AudioPlayerActor {
    pub sink: Sink,
    pub output_stream: OutputStream,
    pub output_stream_handle: OutputStreamHandle,
}

impl Default for AudioPlayerActor {
    fn default() -> Self {
        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&output_stream_handle).unwrap();

        // Don't drop the stream handle for as long as sink lives!
        Self {
            sink,
            output_stream,
            output_stream_handle,
        }
    }
}

impl Actor for AudioPlayerActor {
    type Context = SyncContext<Self>;
}

impl Handler<Audio> for AudioPlayerActor {
    type Result = Result<(), std::io::Error>;

    fn handle(&mut self, msg: Audio, _ctx: &mut SyncContext<Self>) -> Self::Result {
        println!("Actor 3: Received audio data");

        let cursor = Cursor::new(msg.0);
        let source = Decoder::new(cursor).unwrap();

        self.sink.append(source);
        self.sink.sleep_until_end();

        Ok(())
    }
}
