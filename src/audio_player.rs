use std::io::Cursor;

use actix::prelude::*;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct Audio(pub Vec<u8>);

#[derive(Message)]
#[rtype(result = "Result<Status, std::io::Error>")]
pub struct StatusRequest;

#[derive(PartialEq)]
pub enum Status {
    Idle,
    Busy,
}

pub struct AudioPlayerActor {
    sink: Sink,
    // Don't drop the stream and stream handle for as long as the Sink lives!
    #[allow(dead_code)]
    output_stream: OutputStream,
    #[allow(dead_code)]
    output_stream_handle: OutputStreamHandle,
}

impl Default for AudioPlayerActor {
    fn default() -> Self {
        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&output_stream_handle).unwrap();

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
        println!("Audio Player : Received audio data");

        let cursor = Cursor::new(msg.0);
        let source = Decoder::new(cursor).unwrap();

        self.sink.append(source);

        Ok(())
    }
}

impl Handler<StatusRequest> for AudioPlayerActor {
    type Result = Result<Status, std::io::Error>;

    fn handle(&mut self, _msg: StatusRequest, _ctx: &mut SyncContext<Self>) -> Self::Result {
        // println!(
        //     "Audio Player: Received status request. Sink empty? {}",
        //     self.sink.empty()
        // );

        if self.sink.empty() {
            Ok(Status::Idle)
        } else {
            Ok(Status::Busy)
        }
    }
}
