use std::io::{self, Read};
use std::env;
use reqwest::blocking::Client;
use serde_json::json;

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let comment_symbol: char = '#';
    let comment_qtd: usize = 1;
    let ml_comment_symbol: char = '"';
    let ml_comment_qtd: usize = 3;

    let comments = comments_to_json(&input, comment_symbol, comment_qtd, false);
    let ml_comments = comments_to_json(&input, ml_comment_symbol, ml_comment_qtd, true);

    let parsed_json = format!(
        "{{\"comments\": {}, \"ml_comments\": {}}}",
        comments, ml_comments
    );

    let output = check_grammar(&parsed_json).unwrap();

    if output.contains("error") {
        println!("Error: {}", output);
        return;
    }

    print!("{}", output);
}

// Captures comments from a text and returns a JSON object
fn comments_to_json(
    input: &str,
    comment_symbol: char,
    comment_qtd: usize,
    multiline: bool,
) -> String {
    let mut output = String::new();

    output.push_str("{");

    let lines: Vec<String> = input.split("\n").map(String::from).collect();
    let mut lineno = 0;
    let mut capturing = false;

    for line in lines.iter() {
        lineno += 1;

        let mut qtd = 0;
        let mut comment = false;
        let mut column = 0;

        for c in line.chars() {
            column += 1;
            if c == comment_symbol {
                qtd += 1;
                if qtd == comment_qtd {
                    capturing = !capturing && multiline;
                    comment = true;
                    break;
                }
            }
        }

        if comment || capturing {
            let mut new_line = String::new();

            if capturing && !comment {
                new_line.push_str(&line.trim().to_string());
            }

            if !multiline {
                new_line.push_str(&line[column..]);
            }

            if capturing && comment && line.trim().to_string().len() > comment_qtd {
                let clean_line = line.trim().to_string();
                // Remove the comment symbol on init and end
                new_line.push_str(&clean_line[comment_qtd..clean_line.len() - comment_qtd]);
                capturing = false;
            }

            new_line = new_line.trim().to_string();

            if new_line.len() > 0 {
                output.push_str(&format!("\"{}\": \"{}\",", lineno, new_line));
            }
        }
    }

    output.pop();
    output.push_str("}");

    output
}

fn check_grammar(json_data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let openai_token = env::var("OPENAI_API_KEY")?;
    let initial_prompt = "I will send you a JSON with comments of a Python source, check the grammar, ensure that it is straightforward, clear, and concise. You should respond with the same format, the line, and the text. You can add lines if that continues to be clear; for that, you need to insert a new element with the next line number. Give me the lines ordered by line number descending.";

    let url = "https://api.openai.com/v1/chat/completions";
    let client = Client::new();
    
    let res = client
        .post(url)
        .header("Authorization", format!("Bearer {}", openai_token))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": "gpt-4o-mini-2024-07-18",
            "messages": [
                {
                    "role": "system",
                    "content": initial_prompt
                },
                {
                    "role": "user",
                    "content": json_data
                }
            ],
            "max_completion_tokens": 1000,
            "temperature": 0.5,
            "response_format": {"type": "json_object"}
        }))
        .send()?;

    let response_text = res.text()?;

    Ok(response_text)
}
