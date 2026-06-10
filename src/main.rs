use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct ChatResponse {
    response: String,
    #[allow(dead_code)]
    done: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut conversation = String::new();

    println!("Chat Buddy (gemma3:12b) | type 'quit' to exit\n");

    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.eq_ignore_ascii_case("quit") {
            break;
        }

        conversation.push_str(&format!("User: {}\nAssistant:", input));

        let payload = ChatRequest {
            model: "gemma3:12b".into(),
            prompt: conversation.clone(),
            stream: true,
        };

        print!("Buddy: ");
        io::stdout().flush()?;

        let stream = client
            .post("http://localhost:11434/api/generate")
            .json(&payload)
            .send()
            .await?
            .bytes_stream();

        let mut full_response = String::new();

        tokio::pin!(stream);
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            for line in chunk.split(|&b| b == b'\n').filter(|l| !l.is_empty()) {
                if let Ok(resp) = serde_json::from_slice::<ChatResponse>(line) {
                    full_response.push_str(&resp.response);
                    print!("{}", resp.response);
                    io::stdout().flush()?;
                }
            }
        }

        conversation.push_str(&full_response);
        conversation.push('\n');
        println!();
    }

    println!("\nBye!");
    Ok(())
}
