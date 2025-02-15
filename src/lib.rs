pub mod language;
pub mod grammar;

use language::{Comment, CommentType, Language};

use std::env;
use std::collections::HashMap;

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

pub fn handle_args() -> Result<Language, &'static str> {
    let mut args = env::args();

    if args.len() < 2 {
        println!("The --lang attribute is required. (e.g. --lang python)");
        return Err("Language not found");
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
        return Err("Error: Language not supported or not specified.");
    }

    Ok(language.unwrap())
}

fn add_quotes(text: &str) -> String {
    format!("\"{}\"", text.replace("\"", "\\\""))
}

// Captures comments from a text and returns a JSON object
pub fn comments_to_json(comments: &Vec<Comment>) -> String {
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

