use futures::StreamExt;
use tokio::{fs::{write, File}, io::AsyncWriteExt};
use polly::{types::{Engine, OutputFormat}, operation::synthesize_speech::SynthesizeSpeechOutput};
use aws_sdk_polly as polly;


async fn synth(text: &str) {
    let config = aws_config::load_from_env().await;
    let client = polly::Client::new(&config);
    let resp = client
        .synthesize_speech()
        .engine(Engine::Standard)
        .voice_id(polly::types::VoiceId::Miguel)
        .output_format(OutputFormat::Mp3)
        .text(text)
        .send()
        .await
        .unwrap();

    audio_to_file(resp).await;
}

async fn audio_to_file(
    output: SynthesizeSpeechOutput,
) {
    let mut file = File::create("audio.mp3").await.unwrap();
    let mut stream = output.audio_stream;
    while let Some(bytes) = stream.next().await {
        let bytes = bytes.unwrap();
        file.write_all(&bytes).await.unwrap();
    }
    file.flush().await.unwrap();
}