use std::io::{self, Read};

use neospeller::check_spelling;

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

    let output = check_spelling(input, language).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    print!("{}", output);
}
