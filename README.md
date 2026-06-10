# Chat Buddy

A minimal CLI chat agent that streams responses from a local Ollama model (gemma3:12b).

## How this was built (step by step)

### 1. Scaffold the project

```bash
cargo new chat-buddy
cd chat-buddy
```

### 2. Add dependencies

`Cargo.toml` needs four crates:

| Crate | Why |
|-------|-----|
| `reqwest` | HTTP client to talk to Ollama |
| `tokio` | Async runtime (reqwest needs it) |
| `serde` + `serde_json` | Serialize request / deserialize response |
| `futures-util` | `.next()` on the byte stream |

`reqwest` needs the `json` feature for `.json(&payload)` and the `stream` feature to read the response as a byte stream.

### 3. Model the API

Ollama's `/api/generate` endpoint expects:

```json
{ "model": "gemma3:12b", "prompt": "...", "stream": true }
```

And streams back newline-delimited JSON:

```json
{ "response": "Hello", "done": false }
{ "response": " world", "done": false }
{ "response": "!", "done": true }
```

Two `struct`s map these exactly:

```rust
#[derive(Serialize)]
struct ChatRequest { model: String, prompt: String, stream: bool }

#[derive(Deserialize)]
struct ChatResponse { response: String, done: bool }
```

### 4. Make the async call

Wrap `main` in `#[tokio::main]`, then:

```rust
let stream = client
    .post("http://localhost:11434/api/generate")
    .json(&payload)
    .send()
    .await?
    .bytes_stream();
```

`.bytes_stream()` gives a `Stream<Item = Result<Bytes>>` — each item is an HTTP chunk.

### 5. Stream the tokens

Pin the stream and loop with `.next().await`:

```rust
tokio::pin!(stream);
while let Some(chunk) = stream.next().await {
    for line in chunk?.split(|&b| b == b'\n').filter(|l| !l.is_empty()) {
        if let Ok(resp) = serde_json::from_slice::<ChatResponse>(line) {
            print!("{}", resp.response);
        }
    }
}
```

Each chunk may contain multiple JSON lines. Split on `\n`, deserialize, print immediately.

### 6. Maintain conversation context

Append every user message and assistant reply into a `conversation` string, then send the whole history as `prompt`:

```rust
conversation.push_str(&format!("User: {}\nAssistant:", input));
// ... after streaming ...
conversation.push_str(&full_response);
```

This way Gemma sees the full dialogue on each turn.

## Run it

```bash
cargo run --release
```

Type anything. Type `quit` to exit.
