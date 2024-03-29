use actix::{Actor, Handler, Message, SyncContext, Addr};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use rubato::{InterpolationParameters, InterpolationType, Resampler, SincFixedIn, WindowFunction};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::time::{Duration, Instant};
use whisper_rs::{convert_stereo_to_mono_audio, FullParams, SamplingStrategy, WhisperContext};
use anyhow::Result;
use crate::llm::{LlmActor, ChatMessage};

const VOLUME_THRESHOLD: f32 = 0.05;
const SILENCE_DURATION: Duration = Duration::from_secs(1);
const AUDIO_BUFFER: usize = 512;
const OUTPUT_SAMPLE_RATE: usize = 16_000; // as required by Whisper

pub trait GetInput {
    fn record(&mut self) -> String;
}

pub struct Stt {
    ctx: WhisperContext,
    audio_data: Vec<f32>,
    audio_receiver: Receiver<f32>,
    stream: cpal::platform::Stream,
    llm: Addr<LlmActor>,
}

impl Actor for Stt {
    type Context = SyncContext<Self>;
}


#[derive(Message)]
#[rtype(result = "Result<()>")]
pub enum SttAction {
    RecordUntilSilence,
    Pause,
}

impl Handler<SttAction> for Stt {
    type Result = Result<()>;

    fn handle(&mut self, msg: SttAction, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SttAction::RecordUntilSilence => {
                let utterance = self.record();
                self.llm.do_send(ChatMessage(utterance, async_openai::types::Role::User));
            }
            SttAction::Pause => {
                self.stream.pause().expect("Failed to pause stream");
            }
        };

        Ok(())
    }
}

fn audio_input_stream_data_callback(
    n_channels: usize,
    raw_stereo_samples: &[f32],
    tx: &SyncSender<f32>,
    resampler: &mut SincFixedIn<f32>,
) {
    // Convert stereo to mono
    let mut mono_samples = if n_channels == 2 {
        let raw_mono_samples = convert_stereo_to_mono_audio(raw_stereo_samples).unwrap();
        // Resample the audio to get the target sample rate
        resampler.process(&[raw_mono_samples], None).unwrap()
    } else {
        resampler.process(&[raw_stereo_samples], None).unwrap()
    };

    // Send the audio to the main thread
    mono_samples.pop().unwrap().into_iter().for_each(|sample| {
        tx.send(sample)
            .expect("Failed to send audio sample to main thread");
    });
}

fn create_paused_audio_stream(tx: SyncSender<f32>) -> Stream {
    // Get the default host and input device
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("Failed to get default input device");
    println!("Default input device: {:?}", input_device.name());

    // Configure the input stream with default format
    // We want to use the default format
    let input_config: StreamConfig = input_device
        .supported_input_configs()
        .expect("No supported input config found")
        .next()
        .expect("No supported input config found")
        .with_max_sample_rate()
        .into();
    println!("Input config: {:?}", input_config);

    let n_channels = input_config.channels as usize;

    // Create resampler to convert the audio from the input device's sample rate to 16 kHz
    let mut mono_resampler = SincFixedIn::<f32>::new(
        OUTPUT_SAMPLE_RATE as f64 / input_config.sample_rate.0 as f64,
        2.0,
        InterpolationParameters {
            sinc_len: 128,
            f_cutoff: 0.95,
            interpolation: InterpolationType::Linear,
            oversampling_factor: 128,
            window: WindowFunction::BlackmanHarris2,
        },
        AUDIO_BUFFER,
        1,
    )
    .unwrap();

    // Build and play the input stream
    let stream = input_device
        .build_input_stream(
            &input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                audio_input_stream_data_callback(n_channels, data, &tx, &mut mono_resampler);
            },
            move |err| eprintln!("An error occurred on the input audio stream: {}", err),
            None,
        )
        .expect("Failed to build input stream");

    // Initialise with a paused stream
    stream.pause().expect("Failed to pause stream");

    stream
}

impl GetInput for Stt {
    /// Record until no voice activity is detected, then output the text.
    fn record(&mut self) -> String {
        // Start recording
        println!("Audio Player : Start recording");
        self.stream.play().expect("Failed to start recording");

        // Get the audio data from the input stream and run voice activity detection
        self.run_voice_activity_detection();

        // Pause the stream
        self.stream.pause().expect("Failed to pause stream");

        // Not sure how we store this value somewhere in the struct
        // without having to initialise it every time
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
        params.set_n_threads(1);
        params.set_translate(true);
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Run the Whisper ASR model
        println!("Audio Player : Run ASR model");
        self.ctx
            .full(params, &self.audio_data[..])
            .expect("failed to run model");

        // Clear the audio data
        self.audio_data.clear();

        // Fetch the results
        let num_segments = self.ctx.full_n_segments();

        (0..num_segments)
            .map(|i| {
                self.ctx
                    .full_get_segment_text(i)
                    .expect("failed to get segment")
                    .trim()
                    .to_string()
            })
            .filter(|segment| segment != "[BLANK_AUDIO]")
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl Stt {
    pub fn new(path_to_model: String, llm: Addr<LlmActor>) -> Self {
        let ctx = WhisperContext::new(&path_to_model).expect("failed to load model");

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
        params.set_translate(true);
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        let (tx, audio_receiver) = mpsc::sync_channel(AUDIO_BUFFER);

        // Create an audio stream
        let stream = create_paused_audio_stream(tx);

        Self {
            ctx,
            audio_data: Vec::new(),
            audio_receiver,
            stream,
            llm,
        }
    }

    /// Simple voice activity detection using silence duration.
    ///
    /// Note that this function will block the main thread,
    /// while the audio data is being processed concurrently
    /// through the audio input stream
    fn run_voice_activity_detection(&mut self) {
        let mut last_voice_activity = Instant::now();
        while last_voice_activity.elapsed() < SILENCE_DURATION {
            if let Ok(sample) = self.audio_receiver.try_recv() {
                // Check for voice activity
                if sample.abs() > VOLUME_THRESHOLD {
                    last_voice_activity = Instant::now();
                }

                // Add the sample to the audio_data buffer
                self.audio_data.push(sample);
            }
        }
    }
}
