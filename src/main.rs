use dotenvy::dotenv;
use futures_util::StreamExt;
use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use serde::Deserialize;
use serde::Serialize;
use std::{
    env,
    io::{Write, self},
};
use colored::Colorize; // Added for terminal styling

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

    let waiting_for_user_content = create_spinner("I am ready to translate. Please provide the content you need translated.");
    waiting_for_user_content.finish();
    io::stdout().flush()?;

    let mut translate_string = String::new();
    let _ = io::stdin().read_line(&mut translate_string);

    let api_key = env::var("OPENROUTER_API_KEY")
        .map_err(|_| "Error: Missing env field OPENROUTER_API_KEY")?;
    let system_prompty = String::from("Translate user typied content below enclosed in <user_content></user_content> into Simple Chinese, return text only.");

    let model =
        env::var("OPENROUTER_MODEL").map_err(|_| "Error: Missing env field OPENROUTER_MODEL")?;
    let request_body = ChatRequestBody {
        model: model.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompty.to_string()
            },
            Message {
                role: "user".to_string(),
                content: ("<user_content>".to_string() + &translate_string.trim() + "</user_content>").to_string()
            }
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

    while let Some(item) = stream.next().await {
        let chunk = item?;
        let text_chunk = std::str::from_utf8(&chunk)?;

        for line in text_chunk.lines() {
            if line.starts_with("data:") {
                let json_str = line[5..].trim();

                if json_str == "[DONE]" {
                    break;
                }

                if !json_str.is_empty() {
                    match serde_json::from_str::<ApiResponse>(json_str) {
                        Ok(current_item) => {
                            if let Some(choice_data) = current_item.choices.get(0) {
                                print!("{}", choice_data.delta.content.green());
                                io::stdout().flush()?;
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

    Ok(())
}

fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
    );
    pb.set_message(message.to_string());
    pb
}

