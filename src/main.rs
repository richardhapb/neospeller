use reqwest::blocking::Client;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::io::{self, Read};

enum CommentType {
    Single,
    Multi,
}

struct Comment<'a> {
    line: usize,
    text: String,
    comment_type: &'a CommentType,
}

struct Language {
    name: String,
    comment_symbol: String,
    ml_comment_symbol: String,
    ml_comment_symbol_close: String,
}

impl Language {
    fn get_comment_type(&self, line: &str) -> &CommentType {
        if line.find(&self.ml_comment_symbol).is_some() {
            return &CommentType::Multi;
        }

        &CommentType::Single
    }

    fn get_comments(&self, input: &str) -> Vec<Comment> {
        let mut comments = Vec::new();
        let symbol_close = &self.ml_comment_symbol_close;
        let special_chars = vec!['*', '-', '+', '='];

        let lines = input.lines();
        let mut capturing = false;

        for (i, line) in lines.enumerate() {
            let comment_type = if capturing {
                &CommentType::Multi
            } else {
                self.get_comment_type(&line)
            };

            let trimed_line = line.trim();
            let closure = trimed_line.find(symbol_close);

            let symbol = match self.get_comment_type(line) {
                CommentType::Single => &self.comment_symbol,
                CommentType::Multi => &self.ml_comment_symbol,
            };

            if capturing && closure.is_some() {
                capturing = false;

                let closure = closure.unwrap();

                if closure + trimed_line.len() > symbol_close.len() {
                    comments.push(Comment {
                        line: i,
                        text: trimed_line[..closure].trim().to_string(),
                        comment_type,
                    })
                }
                continue;
            }

            if let Some(col) = trimed_line.find(symbol) {
                let comment_length = trimed_line.len();
                let cut = match comment_type {
                    CommentType::Multi => {
                        if comment_length > symbol.len() {
                            capturing = false;
                            let mut cut = 0;
                            for c in special_chars.iter() {
                                if trimed_line[col + symbol.len()..].replace(*c, "").len() == 0 {
                                    capturing = true;
                                    cut = col + symbol.len();
                                    break;
                                }
                            }

                            if cut == 0 {
                                if symbol == symbol_close {
                                    cut = trimed_line.replace(symbol, "").len() + symbol.len();
                                } else {
                                    cut = trimed_line.replace(symbol_close, "").len();
                                }
                            }

                            Some(cut)
                        } else {
                            capturing = true;
                            Some(comment_length)
                        }
                    }

                    CommentType::Single => Some(comment_length),
                };

                let cut = cut.unwrap_or(comment_length);

                let text = trimed_line[col + symbol.len()..cut].trim().to_string();

                if text.len() > 0 {
                    comments.push(Comment {
                        line: i + 1,
                        text,
                        comment_type,
                    });
                }
                continue;
            }

            if capturing {
                comments.push(Comment {
                    line: i + 1,
                    text: trimed_line.to_string(),
                    comment_type,
                });
            }
        }

        comments
    }
}

struct SupportedLanguages {
    languages: Vec<Language>,
}

fn init_supported_languages() -> SupportedLanguages {
    let mut languages = Vec::new();

    let python = Language {
        name: "python".to_string(),
        comment_symbol: "#".to_string(),
        ml_comment_symbol: "\"\"\"".to_string(),
        ml_comment_symbol_close: "\"\"\"".to_string(),
    };

    let javascript = Language {
        name: "javascript".to_string(),
        comment_symbol: "//".to_string(),
        ml_comment_symbol: "/*".to_string(),
        ml_comment_symbol_close: "*/".to_string(),
    };

    let rust = Language {
        name: "rust".to_string(),
        comment_symbol: "//".to_string(),
        ml_comment_symbol: "/*".to_string(),
        ml_comment_symbol_close: "*/".to_string(),
    };

    let css = Language {
        name: "css".to_string(),
        comment_symbol: "//".to_string(),
        ml_comment_symbol: "/*".to_string(),
        ml_comment_symbol_close: "*/".to_string(),
    };

    languages.push(python);
    languages.push(javascript);
    languages.push(rust);
    languages.push(css);

    SupportedLanguages { languages }
}

fn main() {
    let mut args = env::args();

    if args.len() < 2 {
        println!("The --lang attribute is required. (e.g. --lang python)");
        return;
    }

    let mut language: Option<Language> = None;

    while let Some(arg) = args.next() {
        if arg == "--lang" {
            let supported_languages = init_supported_languages();
            let lang = args.next().expect("Language not found (e.g. python)");
            let lang = lang.trim().to_lowercase();

            language = supported_languages
                .languages
                .into_iter()
                .find(|l| l.name == lang);

            break;
        }
    }

    if language.is_none() {
        println!("Error: Language not supported or not specified.");
        return;
    }

    let language = language.unwrap();

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let comments = language.get_comments(&input);
    let parsed_comments = comments_to_json(&comments);

    let output = check_grammar(&parsed_comments, &language.name).unwrap();

    if output.contains("error") {
        println!("Error: {}", output);
        return;
    }

    print!("{}", output);
}

fn add_quotes(text: &str) -> String {
    format!("\"{}\"", text.replace("\"", "\\\""))
}

// Captures comments from a text and returns a JSON object
fn comments_to_json(comments: &Vec<Comment>) -> String {
    let mut output = String::new();
    let mut single_comments: HashMap<String, String> = Default::default();
    let mut ml_comments: HashMap<String, String> = Default::default();

    output.push_str("{");

    for comment in comments.iter() {
        match comment.comment_type {
            CommentType::Single => single_comments.insert(
                add_quotes(&comment.line.to_string()),
                add_quotes(&comment.text.clone()),
            ),
            CommentType::Multi => ml_comments.insert(
                add_quotes(&comment.line.to_string()),
                add_quotes(&comment.text),
            ),
        };
    }

    if single_comments.len() > 0 {
        output.push_str("\"single_comments\": {");
        for (lineno, text) in &single_comments {
            output.push_str(&format!("{}: {},", lineno, text));
        }
        output.pop();
        output.push('}');
    }

    if ml_comments.len() > 0 {
        if single_comments.len() > 0 {
            output.push(',');
        }

        output.push_str("\"multiline_comments\": {");
        for (lineno, text) in &ml_comments {
            output.push_str(&format!("{}: {},", lineno, text));
        }
        output.pop();
        output.push('}');
    }

    output.push_str("}");
    output
}

fn check_grammar(json_data: &str, language: &str) -> Result<String, Box<dyn std::error::Error>> {
    let openai_token = env::var("OPENAI_API_KEY")?;

    let initial_prompt = format!(
        r#"I will send you a JSON containing comments from a {} source file. Your task is to check the grammar and ensure that the comments are straightforward, clear, and concise. Respond in the same JSON format, including the line number and the corrected text.

- You may add new lines if necessary to maintain clarity, but they must be consecutive and properly numbered.
- Do not add periods at the end of lines unless they are necessary for clarity.
- Do not remove formatters such as '-' or '*'; preserve the original formatting and change only the text when necessary.
- Do not change the line numbers for each comment, mantain the original line numbers.
- Do not replace variable names like line_number to line number
- Do not mix single-line comments with multi-line comments; keep them separate."#,
        language
    );

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
            "max_completion_tokens": 2000,
            "temperature": 0.5,
            "response_format": {"type": "json_object"}
        }))
        .send()?;

    let response_text = res.text()?;

    Ok(response_text)
}
