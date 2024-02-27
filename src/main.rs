use clap::{App, Arg};
use reqwest::{self, Body};
use serde_json::{json, Value};
use std::error::Error;

#[derive(Debug)]
struct Conversation {
    messages: Vec<Message>,
}

#[derive(Debug)]
struct Message {
    role: String,
    text: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Command-line argument parsing
    let matches = App::new("Gemini CLI")
        .version("1.0")
        .author("star_evan")
        .arg(
            Arg::new("API_KEY")
                .short('k')
                .long("api-key")
                .value_name("API_KEY")
                .help("Sets the API key for Gemini language model")
                .takes_value(true)
                .default_value("AIzaSyCIC3aAeiISKVaoHvpHExWUkyMgFFGAXqQ"),
        )
        .get_matches();

    // Get API key
    let api_key = matches.value_of("API_KEY").unwrap();

    // Create conversation
    let mut conversation = Conversation {
        messages: Vec::new(),
    };

    // Start conversation loop
    loop {
        // Get user input
        println!("\n\tTo end the conversation, type \":exit\"");
        let user_input = get_user_input();
        if user_input.trim().to_lowercase() == ":exit" {
            break;
        }


        // Add user message to conversation
        conversation.messages.push(Message {
            role: String::from("user"),
            text: user_input.clone(),
        });

        // Build Gemini API request URL
        let api_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}",
            api_key
        );

        // Construct JSON body with conversation history
        let json_body = json!({
            "contents": conversation.messages.iter().map(|msg| {
                json!({
                    "role": &msg.role,
                    "parts": [{"text": &msg.text}],
                })
            }).collect::<Vec<_>>(),
        });

        // Convert JSON to a String and then to a reqwest Body
        let body_str = serde_json::to_string(&json_body)?;
        let body = Body::from(body_str);

        // Send HTTP request
        let client = reqwest::Client::new();
        let response = client
            .post(&api_url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        // Parse JSON response
        let json_response: Value = serde_json::from_str(&response.text().await?)?;

        // Extract and concatenate sentences
        let mut sentence_all = "".to_string();
        if let Some(candidates) = json_response.as_array() {
            for candidate in candidates {
                if let Some(parts) = candidate["candidates"][0]["content"]["parts"].as_array() {
                    let sentence: String = parts
                        .iter()
                        .map(|part| part["text"].as_str().unwrap_or(""))
                        .collect();
                    sentence_all.push_str(&sentence);
                }
            }
        } else if let Some(candidate) = json_response.as_object() {
            if let Some(parts) = candidate["candidates"][0]["content"]["parts"].as_array() {
                let sentence: String = parts
                    .iter()
                    .map(|part| part["text"].as_str().unwrap_or(""))
                    .collect();
                sentence_all.push_str(&sentence);
            }
        }

        // Add model message to conversation
        conversation.messages.push(Message {
            role: String::from("model"),
            text: sentence_all.clone(),
        });

        // Print model's response
        println!("{}", sentence_all);
    }

    Ok(())
}

fn get_user_input() -> String {
    println!("User: ");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}
