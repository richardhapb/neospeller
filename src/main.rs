use std::io::{self, Read};

use neospeller;
use neospeller::grammar;
use neospeller::buffer::Buffer;
use crate::neospeller::language::CommentCollection;

fn main() {

    let language = neospeller::handle_args().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    let language_name = language.name.clone();

    let mut buffer = Buffer::from_string(input, language);
    buffer.get_comments();
    let comments_collection = CommentCollection::from_comments(buffer.comments);
    let parsed_comments = serde_json::to_string(&comments_collection).unwrap();

    let output = grammar::check_grammar(&parsed_comments, &language_name).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    buffer.comments = comments_collection.to_comments();

    buffer.json_to_comments(&output).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    let output = buffer.to_string();


    print!("{}", output);
}
