use dotenvy::dotenv;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use std::{
    env,
    io::{Write, stdout},
};

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

#[derive(Debug, Deserialize)]
struct ApiResponse {
    id: String,
    provider: String,
    model: String,
    object: String,
    created: u64,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    index: u32,
    delta: Delta,
    finish_reason: Option<String>,
    native_finish_reason: Option<String>,
    logprobs: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    role: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("Start--");

    let api_key = env::var("OPENROUTER_API_KEY")
        .map_err(|_| "Error: Missing env field OPENROUTER_API_KEY")?;

    let model =
        env::var("OPENROUTER_MODEL").map_err(|_| "Error: Missing env field OPENROUTER_MODEL")?;
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

    let mut stream = client
        .post(api_url)
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

        for line in text_chunk.lines() {
            if line.starts_with("data:") {
                let json_str = line[5..].trim();

                if json_str == "[DONE]" {
                    println!("\nStream finished");
                    break;
                }

                if !json_str.is_empty() {
                    match serde_json::from_str::<ApiResponse>(json_str) {
                        Ok(current_item) => {
                            if let Some(choice_data) = current_item.choices.get(0) {
                                print!("{}", choice_data.delta.content);
                                stdout().flush()?;
                            }
                            if let Some(choice_data) = current_item.choices.get(0) {
                                if choice_data.finish_reason.is_some() {
                                    println!(
                                        "\nFinished reason: {:?}",
                                        choice_data.finish_reason.as_deref().unwrap_or("")
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("\nJSON Parse failed: {}", e);
                        }
                    }
                }
            }
        }
    }

    println!("End--");
    Ok(())
}
