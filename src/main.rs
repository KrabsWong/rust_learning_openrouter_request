use reqwest::Client;
use serde::Serialize;
use std::{env, io::{stdout, Write}};
use futures_util::StreamExt;
use dotenvy::dotenv;

// 定义消息结构体
#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequestBody {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("Start--");

    let api_key = env::var("OPENROUTER_API_KEY")
        .map_err(|_| "Error: Missing env field OPENROUTER_API_KEY")?;

    let model = env::var("OPENROUTER_MODEL")
        .map_err(|_| "Error: Missing env field OPENROUTER_MODEL")?;
    let request_body = ChatRequestBody {
        model: model.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "your are a helpful assitant".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "Tell me something about Rust".to_string(),
            },
        ],
        stream: true,
    };

    let client = Client::new();
    let api_url = "https://openrouter.ai/api/v1/chat/completions";

    let mut stream = client.post(api_url)
        .bearer_auth(&api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?
        .error_for_status()?
        .bytes_stream();

    println!("Get the stream response");
    while let Some(item) = stream.next().await {
        let chunk = item?;
        let text_chunk = std::str::from_utf8(&chunk)?;

        print!("{}", text_chunk);

        stdout().flush()?;
    }

    println!("End--");
    Ok(())
}
