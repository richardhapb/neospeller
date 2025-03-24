pub mod buffer;
pub mod grammar;
pub mod language;

use language::{init_supported_languages, Language, CommentCollection};
use buffer::{Buffer, sort_comments_by_line_number};

use std::env;

/// Handle the CLI args
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

            language = supported_languages.languages.into_iter().find(|l| l.name == lang);

            break;
        }
    }

    if language.is_none() {
        return Err("Error: Language not supported or not specified.");
    }

    Ok(language.unwrap())
}

/// Main entry point for the spell checker
/// 
/// # Arguments
/// 
/// * `input` - The source code to check
/// * `language` - The programming language of the source code
/// 
/// # Returns
/// 
/// * The corrected source code
pub fn check_spelling(input: String, language: Language) -> Result<String, Box<dyn std::error::Error>> {
    let language_name = language.name.clone();

    let mut buffer = Buffer::from_string(input, language);
    buffer.get_comments();
    let comments_collection = CommentCollection::from_comments(buffer.comments);
    let parsed_comments = serde_json::to_string(&comments_collection)?;

    let output = grammar::check_grammar(&parsed_comments, &language_name)?;

    buffer.comments = comments_collection.to_comments();
    buffer.comments = sort_comments_by_line_number(buffer.comments);

    buffer.json_to_comments(&output)?;

    Ok(buffer.to_string())
}
