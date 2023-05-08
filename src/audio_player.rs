use std::{io::Cursor, time::Duration};

use actix::prelude::*;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

pub struct Audio(pub Vec<u8>);

impl Message for Audio {
    type Result = Result<(), std::io::Error>;
}

pub struct AudioPlayerActor {
    pub sink: Sink,
    pub output_stream: OutputStream,
    pub output_stream_handle: OutputStreamHandle,
    pub remaining_duration: Duration,
}

impl Default for AudioPlayerActor {
    fn default() -> Self {
        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&output_stream_handle).unwrap();
        let remaining_duration = Duration::ZERO;

        // Don't drop the stream handle for as long as sink lives!
        Self {
            sink,
            output_stream,
            output_stream_handle,
            remaining_duration,
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

        let num_samples = msg.0.len();
        let cursor = Cursor::new(msg.0);
        let source = Decoder::new(cursor).unwrap();

        let est_duration = calculate_duration(num_samples, source.sample_rate(), source.channels(), 16);

        self.sink.append(source);
        self.sink.sleep_until_end();

        self.remaining_duration += est_duration;

        println!("Actor 3: Remaining: {:?}", self.remaining_duration);

        Ok(())
    }
}

fn calculate_duration(data_size: usize, sample_rate: u32, channels: u16, bits_per_sample: u16) -> Duration {
    let bytes_per_sample = bits_per_sample / 8;
    let total_samples = data_size / (bytes_per_sample as usize * channels as usize);
    let duration_secs = total_samples as f32 / (sample_rate as f32 * channels as f32);
    Duration::from_secs_f32(duration_secs)
}
