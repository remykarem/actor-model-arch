Turn-based chat interface

Stack
* LLM: ChatGPT
* Speech-to-text: Whisper
* Text-to-speech: Amazon Polly
* Actor Model framework: Actix
* Audio playback library: rodio
* Audio input library: cpal


1. Download Protobuf.

    ```sh
    brew install protobuf
    ```

    Protobuf is needed for gRPC with the vector database.

2. Download libtorch v1.13.1 for macOS CPU from [here](https://download.pytorch.org/libtorch/cpu/libtorch-macos-1.13.1.zip). Then unzip it.

3. Set the following env variables:
  
    ```sh
    export LIBTORCH=<path-to-unzipped-folder>
    export DYLD_FALLBACK_LIBRARY_PATH=${LIBTORCH}/lib  # for macOS
    ```

    For other platforms, see [here](https://doc.rust-lang.org/cargo/reference/environment-variables.html#dynamic-library-paths) to specify dynamic library path.

3. Run the Qdrant container.

    ```sh
    docker run -d -p 6333:6333 -p 6334:6334 \
        -v $(pwd)/qdrant_storage:/qdrant/storage \
        qdrant/qdrant
    ```

4. Run

    ```sh
    cargo run
    ```

VS Code

```
"rust-analyzer.cargo.extraEnv": {
    "LIBTORCH": "/root/miniconda/lib/python3.10/site-packages/torch",
    "LD_LIBRARY_PATH": "/root/miniconda/lib/python3.10/site-packages/torch/lib",
    "DYLD_FALLBACK_LIBRARY_PATH": "/root/miniconda/lib/python3.10/site-packages/torch/lib",
    "LIBTORCH_CXX11_ABI": "0"
}
```
