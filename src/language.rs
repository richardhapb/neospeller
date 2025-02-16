#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommentType {
    Single,
    Multi,
}

#[derive(Debug)]
pub struct ParseState {
    pub comments: Vec<Comment>,
    pub lines_parsed: usize,
}

#[derive(Debug)]
pub struct Comment {
    pub line: usize,
    pub text: String,
    pub comment_type: CommentType,
}

impl Comment {
    pub fn new(line: usize, text: String, comment_type: CommentType) -> Comment {
        Comment {
            line,
            text,
            comment_type,
        }
    }

    pub fn parse_comment(
        language: &Language,
        input: &str,
        start_line: usize,
        comment_type: CommentType,
    ) -> Result<ParseState, &'static str> {
        let mut comments = Vec::new();
        let mut lines_parsed = 0;

        let lines: Vec<&str> = input.lines().collect();
        if lines.is_empty() {
            return Err("Empty input");
        }

        match comment_type {
            CommentType::Single => {
                if let Some(comment) = parse_single_line_comment(language, lines[0], start_line) {
                    comments.push(comment);
                    lines_parsed = 1;
                }
            }
            CommentType::Multi => {
                if let Some(parse_state) =
                    parse_multi_line_comment(language, &lines, start_line, comment_type)
                {
                    comments.extend(parse_state.comments);
                    lines_parsed = parse_state.lines_parsed;
                }
            }
        }

        Ok(ParseState {
            comments,
            lines_parsed,
        })
    }
}

fn parse_single_line_comment<'a>(
    language: &Language,
    line: &str,
    line_number: usize,
) -> Option<Comment> {
    if let Some(pos) = line.find(&language.comment_symbol) {
        let comment_text = line[pos + language.comment_symbol.len()..].trim();
        if !comment_text.is_empty() {
            return Some(Comment::new(
                line_number,
                comment_text.to_string(),
                CommentType::Single,
            ));
        }
    }
    None
}

fn parse_multi_line_comment(
    language: &Language,
    lines: &[&str],
    start_line: usize,
    comment_type: CommentType,
) -> Option<ParseState> {
    let mut comments = Vec::new();
    let mut lines_parsed = 0;
    let mut found_close = false;

    let first_line = lines[0];
    if let Some(start_pos) = first_line.find(&language.ml_comment_symbol) {
        let mut text = first_line[start_pos + language.ml_comment_symbol.len()..].trim();

        // Handle single-line multi-line comment
        if let Some(end_pos) = text.find(&language.ml_comment_symbol_close) {
            text = &text[..end_pos].trim();
            if !text.is_empty() {
                comments.push(Comment::new(start_line, text.to_string(), comment_type));
            }
            lines_parsed = 1;
            return Some(ParseState {
                comments,
                lines_parsed,
            });
        }

        // Process multi-line comment
        if !text.is_empty() {
            comments.push(Comment::new(start_line, text.to_string(), comment_type));
        }

        for (i, &line) in lines[1..].iter().enumerate() {
            lines_parsed = i + 2;
            let trimmed = line.trim();

            if let Some(end_pos) = trimmed.find(&language.ml_comment_symbol_close) {
                let comment_text = trimmed[..end_pos].trim();
                if !comment_text.is_empty() {
                    comments.push(Comment::new(
                        start_line + i + 1,
                        comment_text.to_string(),
                        comment_type,
                    ));
                }
                found_close = true;
                break;
            }

            if !trimmed.is_empty() {
                comments.push(Comment::new(
                    start_line + i + 1,
                    trimmed.to_string(),
                    comment_type,
                ));
            }
        }

        if found_close {
            return Some(ParseState {
                comments,
                lines_parsed,
            });
        }
    }
    None
}

pub struct Language {
    pub name: String,
    pub comment_symbol: String,
    pub ml_comment_symbol: String,
    pub ml_comment_symbol_close: String,
}

impl Language {
    pub fn get_comment_type(&self, line: &str) -> CommentType {
        // First check for multi-line comment
        if let Some(ml_pos) = line.find(&self.ml_comment_symbol) {
            // Make sure it's not inside a string
            let before = &line[..ml_pos];
            let quotes = before.chars().filter(|&c| c == '"' || c == '\'').count();
            if quotes % 2 == 0 {
                return CommentType::Multi;
            }
        }
        CommentType::Single
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_comment_type() {
        let language = Language {
            name: "rust".to_string(),
            comment_symbol: "//".to_string(),
            ml_comment_symbol: "/*".to_string(),
            ml_comment_symbol_close: "*/".to_string(),
        };

        let single_line = "let x = 5; // this is a comment";
        let multi_line = "/* this is a\nmulti-line comment */";

        assert_eq!(language.get_comment_type(single_line), CommentType::Single);
        assert_eq!(language.get_comment_type(multi_line), CommentType::Multi);
    }
}
