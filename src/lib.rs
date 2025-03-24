pub mod buffer;
pub mod grammar;
pub mod language;

use language::{init_supported_languages, Language};

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
