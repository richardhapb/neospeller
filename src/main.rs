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

    let buffer = Buffer::from_string(input);

    let comments = language.get_comments(&buffer);
    let parsed_comments = neospeller::comments_to_json(&comments);

    let output = grammar::check_grammar(&parsed_comments, &language.name).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    if output.contains("error") {
        println!("Error: {}", output);
        return;
    }

    print!("{}", output);
}
