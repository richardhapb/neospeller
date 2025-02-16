use std::io::{self, Read};

use neospeller;
use neospeller::grammar;
use neospeller::buffer::Buffer;

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

    let mut buffer = Buffer::from_string(input);

    buffer.get_comments(&language);
    let parsed_comments = buffer.comments_to_json();

    let output = grammar::check_grammar(&parsed_comments, &language.name).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    if output.contains("error") {
        println!("Error: {}", output);
        return;
    }

    buffer.json_to_comments(&output, &language).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    let output = buffer.to_string();


    print!("{}", output);
}
