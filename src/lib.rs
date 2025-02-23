pub mod language;
pub mod grammar;
pub mod buffer;

use language::{Language, init_supported_languages};

use std::env;

pub fn handle_args() -> Result<Language, &'static str> {
    let mut args = env::args();

    if args.len() < 2 {
        eprintln!("The --lang attribute is required. (e.g. --lang python)");
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
    let special_chars = vec!["\"", "*"];
    let mut final_text = text.to_string();

    for char in special_chars {
        final_text = final_text.replace(char, &format!("\\{}", char));
    }
    format!("\"{}\"", final_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use language::{Comment, CommentType};
    use buffer::Buffer;

    #[test]
    fn test_add_quotes() {
        let text = "124";
        let quoted_text = add_quotes(text);
        assert_eq!(quoted_text, "\"124\"");
    }

    #[test]
    fn test_comments_to_json() {
        let comments = vec![
            Comment {
                line: 1,
                text: "A class that represents a HttpRequest".to_string(),
                comment_type: CommentType::Single,
            },
            Comment {
                line: 122,
                text: "Args:".to_string(),
                comment_type: CommentType::Multi,
            },
            Comment {
                line: 124,
                text: "count -> int: The counter of a loop".to_string(),
                comment_type: CommentType::Multi,
            },
        ];

        let mut buffer = Buffer::new();

        buffer.comments = comments;

        let json = buffer.comments_to_json();
        let expected = r#"{"single_comments": {"1": "A class that represents a HttpRequest"},"multiline_comments": {"122": "Args:","124": "count -> int: The counter of a loop"}}"#;
        assert_eq!(json, expected);
    }
}

