use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let comment_symbol: char = '#';
    let comment_qtd: usize = 1;
    let ml_comment_symbol: char = '"';
    let ml_comment_qtd: usize = 3;

    let comments = comments_to_json(&input, comment_symbol, comment_qtd, false);
    let ml_comments = comments_to_json(&input, ml_comment_symbol, ml_comment_qtd, true);

    let output = format!(
        "{{\"comments\": {}, \"ml_comments\": {}}}",
        comments, ml_comments
    );

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
                // Remove the comment symbol on init and end
                new_line.push_str(&line.trim().to_string()[comment_qtd..line.len() - comment_qtd]);
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
