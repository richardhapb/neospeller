use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main type that represents single line comment
/// or multiline comment
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum CommentType {
    #[serde(rename = "single_comments")]
    Single,
    #[serde(rename = "multiline_comments")]
    Multi,
}

impl CommentType {
    /// Convert a [`str`] to [`CommentType`]
    pub fn from(string: &str) -> Result<CommentType, String> {
        match string {
            "single_comments" => Ok(CommentType::Single),
            "multiline_comments" => Ok(CommentType::Multi),
            _ => Err("Invalid comment type".to_string()),
        }
    }

    /// Convert a [`CommentType`] to [`str`]
    pub fn as_str(&self) -> &str {
        match self {
            CommentType::Single => "single_comments",
            CommentType::Multi => "multiline_comments",
        }
    }
}

/// The parsing state that is a response of parsing,
/// allow handle the offset and the comments from text
#[derive(Debug)]
pub struct ParseState {
    pub comments: Vec<Comment>,
    pub lines_parsed: usize,
}

/// Main structure that represents a comment
#[derive(Debug)]
pub struct Comment {
    pub line: usize,
    pub text: String,
    pub comment_type: CommentType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentCollection {
    single_comments: HashMap<usize, String>,
    multiline_comments: HashMap<usize, String>,
}

impl CommentCollection {
    pub fn from_comments(comments: Vec<Comment>) -> Self {
        let mut single_comments = HashMap::new();
        let mut multiline_comments = HashMap::new();

        for comment in comments {
            match comment.comment_type {
                CommentType::Single => single_comments.insert(comment.line, comment.text),
                CommentType::Multi => multiline_comments.insert(comment.line, comment.text),
            };
        }

        Self {
            single_comments,
            multiline_comments,
        }
    }

    pub fn to_comments(&self) -> Vec<Comment> {
        let mut comments: Vec<Comment> = vec![];
        for (line, text) in self.single_comments.iter() {
            comments.push(Comment {
                line: *line,
                text: text.to_string(),
                comment_type: CommentType::Single,
            });
        }

        for (line, text) in self.multiline_comments.iter() {
            comments.push(Comment {
                line: *line,
                text: text.to_string(),
                comment_type: CommentType::Multi,
            });
        }

        comments
    }
}

impl Comment {
    /// Create a new [`Comment`]
    pub fn new(line: usize, text: String, comment_type: CommentType) -> Comment {
        Comment {
            line,
            text,
            comment_type,
        }
    }

    /// Retrieve comments from provided text
    ///
    /// # Params
    /// * `language`: [`Language`] instance of the text's language
    /// * `text`: Text to parse
    /// * `start_line`: The first line where begin parsing
    /// * `comment_type`: [`CommentType`] instance, single line or multiline
    ///
    /// # Returns
    /// * [`ParseState`] instance representing lines parsed and comments retrieved
    /// * str if an error occurred
    pub fn parse_comment(
        language: &Language,
        lines: &[String],
        start_line: usize,
        comment_type: CommentType,
    ) -> Result<ParseState, &'static str> {
        let mut comments = Vec::new();
        let mut lines_parsed = 0;

        if lines.is_empty() {
            return Err("Empty input");
        }

        match comment_type {
            CommentType::Single => {
                if let Some(comment) = parse_single_line_comment(language, &lines[0], start_line) {
                    comments.push(comment);
                    lines_parsed = 1; //Always is one line
                }
            }
            CommentType::Multi => {
                if let Some(parse_state) = parse_multi_line_comment(language, lines, start_line) {
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

/// Parse a single line comment from provided line
///
/// # Params
/// * `language`: [`Language`] instance of the text's language
/// * `line`: Line to parse
/// * `line_number`: Number of the line provided
///
/// # Returns
/// * [`Comment`] instance if comment has been parsed or `None`
fn parse_single_line_comment(language: &Language, line: &str, line_number: usize) -> Option<Comment> {
    if let Some(pos) = line.find(&language.comment_symbol) {
        let comment_text = line[pos + language.comment_symbol.len()..].trim();

        // Ensure that the quantity of quotes is not odd,
        // that could indicate that the symbol is enclosed in quotes
        let before = &line[..pos];
        let quotes = before.chars().filter(|&c| c == '"' || c == '\'').count();

        if !comment_text.is_empty() && quotes % 2 == 0 {
            return Some(Comment::new(
                line_number,
                comment_text.to_string(),
                CommentType::Single,
            ));
        }
    }
    None
}

/// Parse a multi-line comment from provided line
///
/// # Params
/// * `language`: [`Language`] instance of the text's language
/// * `lines`: Lines to parse
/// * `start_line`: Number of the line where comment begins
///
/// # Returns
/// * [`ParseState`] instance with the comments and lines parsed
fn parse_multi_line_comment(language: &Language, lines: &[String], start_line: usize) -> Option<ParseState> {
    let mut comments = Vec::new();
    let comment_type = CommentType::Multi;

    let first_line = &lines[0];
    if let Some(start_pos) = first_line.find(&language.ml_comment_symbol) {
        let mut lines_parsed = 1; // Always parse almost one line
        let mut text = first_line[start_pos + language.ml_comment_symbol.len()..].trim();

        // Handle single-line multi-line comment for example in `python`:
        // """Single line comment in Python using multi-line symbol"""
        if let Some(end_pos) = text.find(&language.ml_comment_symbol_close) {
            text = text[..end_pos].trim();
            if !text.is_empty() {
                comments.push(Comment::new(start_line, text.to_string(), comment_type));
            }
            return Some(ParseState {
                comments,
                lines_parsed,
            });
        }

        // Process "real" multi-line comment

        if !text.is_empty() {
            // In case of begin with symbol but has line breaks, like:
            // """Comment in multi-line
            // using symbol in same line"""
            comments.push(Comment::new(start_line, text.to_string(), comment_type));
        }

        for (i, line) in lines[1..].iter().enumerate() {
            lines_parsed += 1;
            let text = line.trim().to_string();

            // Last line
            if let Some(end_pos) = text.find(&language.ml_comment_symbol_close) {
                let text = text[..end_pos].trim().to_string();
                if !text.is_empty() {
                    comments.push(Comment::new(start_line + i + 1, text, comment_type));
                }
                break;
            }

            comments.push(Comment::new(start_line + i + 1, text, comment_type));
        }

        return Some(ParseState {
            comments,
            lines_parsed,
        });
    }
    // If the opening or closing symbol is not found returns None
    None
}

/// Language parameters
pub struct Language {
    pub name: String,
    pub comment_symbol: String,
    pub ml_comment_symbol: String,
    pub ml_comment_symbol_close: String,
}

impl Language {
    /// Get comment type depending on symbol
    /// by default returns single line comment
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

/// Languages parameters configuration
pub struct SupportedLanguages {
    pub languages: Vec<Language>,
}

/// Languages configuration
pub fn init_supported_languages() -> SupportedLanguages {
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

    let lua = Language {
        name: "lua".to_string(),
        comment_symbol: "--".to_string(),
        ml_comment_symbol: "--[[".to_string(),
        ml_comment_symbol_close: "]]".to_string(),
    };

    let c = Language {
        name: "c".to_string(),
        comment_symbol: "//".to_string(),
        ml_comment_symbol: "/*".to_string(),
        ml_comment_symbol_close: "*/".to_string(),
    };

    let bash = Language {
        name: "bash".to_string(),
        comment_symbol: "#".to_string(),
        ml_comment_symbol: ": '".to_string(),
        ml_comment_symbol_close: "'".to_string(),
    };

    let text = Language {
        name: "text".to_string(),
        comment_symbol: "".to_string(),
        ml_comment_symbol: "".to_string(),
        ml_comment_symbol_close: "".to_string(),
    };

    languages.push(python);
    languages.push(javascript);
    languages.push(rust);
    languages.push(css);
    languages.push(lua);
    languages.push(c);
    languages.push(bash);
    languages.push(text);

    SupportedLanguages { languages }
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
