use std::fmt::Display;

use crate::language::{Comment, CommentCollection, CommentType, Language};

/// Text Buffer
pub struct Buffer {
    pub lines: Vec<String>,
    pub comments: Vec<Comment>,
    pub language: Language,
}

impl Buffer {
    /// Create a new [`Buffer`]
    pub fn new(language: Language) -> Self {
        Self {
            lines: Vec::new(),
            comments: Vec::new(),
            language,
        }
    }

    /// Create a new buffer from string
    pub fn from_string(s: String, language: Language) -> Self {
        let lines: Vec<String> = s.lines().map(|l| l.to_string()).collect();
        Self {
            lines,
            comments: Vec::new(),
            language,
        }
    }

    /// Add a line to [`Buffer`]
    pub fn push(&mut self, line: String) {
        self.lines.push(line);
    }

    /// Get comments from text of the buffer
    pub fn get_comments(&mut self) -> &Vec<Comment> {
        let mut comments = Vec::new();
        let mut i = 0;

        while i < self.lines.len() {
            let line = &self.lines[i];

            // Skip empty lines
            if line.trim().is_empty() {
                i += 1;
                continue;
            }

            let comment_type = self.language.get_comment_type(line);

            // Attempt to parse the comment starting at the current line
            if let Ok(parse_state) =
                Comment::parse_comment(&self.language, &self.lines[i..], i, comment_type)
            {
                if parse_state.lines_parsed > 0 {
                    comments.extend(parse_state.comments);
                    i += parse_state.lines_parsed;
                    continue;
                }
            }
            i += 1;
        }

        self.comments = comments;
        &self.comments
    }

    /// Replace comments in text, overwrite old comments with fixed comments
    ///
    /// # Params
    /// * `new_comments`: A [`Comment`] vector with new comments to replace
    ///
    /// # Returns
    /// * Error it the comment cannot be replaced
    pub fn replace_comments(&mut self, new_comments: &[Comment]) -> Result<(), &'static str> {
        for (i, comment) in new_comments.iter().enumerate() {
            let line = self.lines.get_mut(comment.line).ok_or("Line not found")?;

            let new_line = match comment.comment_type {
                CommentType::Single => replace_single_comment(line, &self.comments[i].text, &comment.text),
                CommentType::Multi => {
                    replace_multi_comment(line, &self.comments[i].text, &comment.text, &self.language)
                }
            };

            *line = new_line?;
        }

        Ok(())
    }

    /// Convert a json to buffer's comments and order by line number
    ///
    /// # Params
    /// * `json_string`: Json to convert
    pub fn json_to_comments(&mut self, json_string: &str) -> Result<&Vec<Comment>, &'static str> {
        let comments: CommentCollection =
            serde_json::from_str(json_string).map_err(|_| "Error parsing json string")?;

        let comments = sort_comments_by_line_number(comments.to_comments());
        self.replace_comments(&comments)?;
        self.comments = comments;

        Ok(&self.comments)
    }
}

impl Display for Buffer {
    /// Convert [`Buffer`] to a string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}

/// Orders comments by line number using an efficient sort
///
/// # Arguments
/// * `comments` - Vector of Comment structures to be sorted
///
/// # Returns
/// * Sorted vector of comments by line number
///
/// # Performance
/// * Time complexity: O(n log n)
/// * Space complexity: O(1) as it sorts in place
pub fn sort_comments_by_line_number(mut comments: Vec<Comment>) -> Vec<Comment> {
    comments.sort_by_key(|comment| comment.line);
    comments
}

/// Replace a single line comment
///
/// # Params
/// * `line`: Line where comment is located
/// * `old_comment`: Old comment text
/// * `new_comment`: New comment text
///
/// # Returns
/// * The new line text or an Error if it cannot be replaced
fn replace_single_comment(
    line: &mut str,
    old_comment: &str,
    new_comment: &str,
) -> Result<String, &'static str> {
    let mut result = String::new();

    if let Some(sym_index) = line.find(old_comment) {
        if sym_index > 0 {
            result.push_str(&line[..sym_index]);
        }

        result.push_str(new_comment);
        return Ok(result);
    }

    Err("No comment found")
}

/// Replace a multi line comment
///
/// # Params
/// * `line`: Line where comment is located
/// * `old_comment`: Old comment text
/// * `new_comment`: New comment text
///
/// # Returns
/// * The new line text or an Error if it cannot be replaced
fn replace_multi_comment(
    line: &mut str,
    old_comment: &str,
    new_comment: &str,
    language: &Language,
) -> Result<String, &'static str> {
    let mut result = String::new();

    if let Some(sym_index) = line.find(old_comment) {
        if sym_index > 0 {
            result.push_str(&line[..sym_index]);
        }

        result.push_str(new_comment);

        if line[sym_index..].contains(&language.ml_comment_symbol_close) {
            if sym_index > 0 && line.as_bytes()[sym_index - 1] == b' ' {
                result.push(' ');
            }

            result.push_str(&language.ml_comment_symbol_close);
        }

        return Ok(result);
    }

    Err("No comment found")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const RUST_FIXTURE: &str = r#"// this is a single line comment
let x = 5;

/*
this is a
multi-line comment
*/

/* Another multi-line comment, but in a single line */

/* Unestruturated multi-line comment
*/

    /* 
    Another unestruturated multi-line comment 
    With multiples lines */

foo();

bar = 5;

/// * Documentation code
        "#;

    const PYTHON_FIXTURE: &str = r#""""
this is a
multi-line comment
"""

# this is a single line comment
x = 5


"""Another multi-line comment, but in a single line"""

""" Unestruturated multi-line comment
"""

""" 
Another unestruturated multi-line comment 
With multiples lines """

foo()

bar = 5

string = "hello"
another = 'hello'

    # Print debug information to compare with the visual content in the browser and verify the order.
    # Profiles online should be in the positions: [7, 57] and [3, 15, 17] according to the get_profiles_display_group_settings function.
    # If you change the initial online IDs, another filter may capture them first. Check if this occurs.
    # print(f"profile_list[{position}]: {profiles_list[position]}")



CONSTANT = 5
""" last """
""" comment """
"#;

    #[test]
    fn test_get_comments_rust() {
        let language = Language {
            name: "rust".to_string(),
            comment_symbol: "//".to_string(),
            ml_comment_symbol: "/*".to_string(),
            ml_comment_symbol_close: "*/".to_string(),
        };

        let mut buffer = Buffer::from_string(RUST_FIXTURE.to_string(), language);
        let comments = buffer.get_comments();

        assert_eq!(comments.len(), 8);

        assert_eq!(comments[0].line, 0);
        assert_eq!(comments[0].text, "this is a single line comment");
        assert_eq!(comments[0].comment_type, CommentType::Single);

        assert_eq!(comments[1].line, 4);
        assert_eq!(comments[1].text, "this is a");
        assert_eq!(comments[1].comment_type, CommentType::Multi);

        assert_eq!(comments[2].text, "multi-line comment");
        assert_eq!(comments[2].comment_type, CommentType::Multi);
        assert_eq!(comments[2].line, 5);

        assert_eq!(
            comments[3].text,
            "Another multi-line comment, but in a single line"
        );
        assert_eq!(comments[3].line, 8);
        assert_eq!(comments[3].comment_type, CommentType::Multi);

        assert_eq!(comments[4].text, "Unestruturated multi-line comment");
        assert_eq!(comments[4].line, 10);
        assert_eq!(comments[4].comment_type, CommentType::Multi);

        assert_eq!(comments[5].text, "Another unestruturated multi-line comment");
        assert_eq!(comments[5].line, 14);
        assert_eq!(comments[5].comment_type, CommentType::Multi);

        assert_eq!(comments[6].text, "With multiples lines");
        assert_eq!(comments[6].line, 15);
        assert_eq!(comments[6].comment_type, CommentType::Multi);

        assert_eq!(comments[7].text, "/ * Documentation code");
        assert_eq!(comments[7].line, 21);
        assert_eq!(comments[7].comment_type, CommentType::Single);
    }

    #[test]
    fn test_get_comments_python() {
        let language = Language {
            name: "python".to_string(),
            comment_symbol: "#".to_string(),
            ml_comment_symbol: "\"\"\"".to_string(),
            ml_comment_symbol_close: "\"\"\"".to_string(),
        };

        let mut buffer = Buffer::from_string(PYTHON_FIXTURE.to_string(), language);
        let comments = buffer.get_comments();

        assert_eq!(comments.len(), 13);

        assert_eq!(comments[0].line, 1);
        assert_eq!(comments[0].text, "this is a");
        assert_eq!(comments[0].comment_type, CommentType::Multi);

        assert_eq!(comments[1].text, "multi-line comment");
        assert_eq!(comments[1].comment_type, CommentType::Multi);
        assert_eq!(comments[1].line, 2);

        assert_eq!(comments[2].line, 5);
        assert_eq!(comments[2].text, "this is a single line comment");
        assert_eq!(comments[2].comment_type, CommentType::Single);

        assert_eq!(
            comments[3].text,
            "Another multi-line comment, but in a single line"
        );
        assert_eq!(comments[3].line, 9);
        assert_eq!(comments[3].comment_type, CommentType::Multi);

        assert_eq!(comments[4].text, "Unestruturated multi-line comment");
        assert_eq!(comments[4].line, 11);
        assert_eq!(comments[4].comment_type, CommentType::Multi);

        assert_eq!(comments[5].text, "Another unestruturated multi-line comment");
        assert_eq!(comments[5].line, 15);
        assert_eq!(comments[5].comment_type, CommentType::Multi);

        assert_eq!(comments[6].text, "With multiples lines");
        assert_eq!(comments[6].line, 16);
        assert_eq!(comments[6].comment_type, CommentType::Multi);

        assert_eq!(
            comments[7].text,
            "Print debug information to compare with the visual content in the browser and verify the order."
        );
        assert_eq!(comments[8].text, "Profiles online should be in the positions: [7, 57] and [3, 15, 17] according to the get_profiles_display_group_settings function.");
        assert_eq!(comments[9].text, "If you change the initial online IDs, another filter may capture them first. Check if this occurs.");
        assert_eq!(
            comments[10].text,
            "print(f\"profile_list[{position}]: {profiles_list[position]}\")"
        );
        assert_eq!(comments[11].text, "last");
        assert_eq!(comments[12].text, "comment");
    }

    #[test]
    fn test_replace_comments() {
        let language = Language {
            name: "python".to_string(),
            comment_symbol: "#".to_string(),
            ml_comment_symbol: "\"\"\"".to_string(),
            ml_comment_symbol_close: "\"\"\"".to_string(),
        };

        let mut buffer = Buffer::from_string(PYTHON_FIXTURE.to_string(), language);
        let comments = buffer.get_comments();

        let mut new_comments = vec![];

        for comment in comments.iter() {
            new_comments.push(Comment {
                line: comment.line,
                text: comment.text.replace('a', "e"),
                comment_type: comment.comment_type,
            })
        }

        buffer.replace_comments(&new_comments).unwrap();

        assert_eq!(buffer.lines[0].trim(), "\"\"\"");
        assert_eq!(buffer.lines[1].trim(), "this is e");
        assert_eq!(buffer.lines[2].trim(), "multi-line comment"); // Not change
        assert_eq!(buffer.lines[3].trim(), "\"\"\"");
        assert_eq!(buffer.lines[4].trim(), "");
        assert_eq!(buffer.lines[5].trim(), "# this is e single line comment");
        assert_eq!(
            buffer.lines[9].trim(),
            "\"\"\"Another multi-line comment, but in e single line\"\"\""
        );
    }

    #[test]
    fn test_json_to_comments() {
        let json_string = r#"{"single_comments": {"1": "A class that represents a HttpRequest"},"multiline_comments": {"122": "Args:","124": "count -> int: The counter of a loop"}}"#;
        let comments = serde_json::from_str::<CommentCollection>(json_string)
            .unwrap()
            .to_comments();

        // Create a map for easier verification
        let comments_map: HashMap<_, _> = comments
            .iter()
            .map(|c| (c.line, (c.text.as_str(), c.comment_type)))
            .collect();

        assert_eq!(
            comments_map.get(&1),
            Some(&("A class that represents a HttpRequest", CommentType::Single))
        );
        assert_eq!(comments_map.get(&122), Some(&("Args:", CommentType::Multi)));
        assert_eq!(
            comments_map.get(&124),
            Some(&("count -> int: The counter of a loop", CommentType::Multi))
        );

        let json_string = r#"{  "single_comments": {    "42": "Another comment with spelling mistakes?"  },  "multiline_comments": {    "4": "Docstring of a function",    "6": "Args:",    "7": "dictarg (dict): A dictionary argument",    "19": "Multiline comment that is not a docstring",    "26": "Alice's Adventures in Wonderland by Lewis Carroll",    "27": "A Project Gutenberg eBook"  }}"#;
        let comments = serde_json::from_str::<CommentCollection>(json_string)
            .unwrap()
            .to_comments();

        let comments_map: HashMap<_, _> = comments
            .iter()
            .map(|c| (c.line, (c.text.as_str(), c.comment_type)))
            .collect();

        assert_eq!(
            comments_map.get(&42),
            Some(&("Another comment with spelling mistakes?", CommentType::Single))
        );
        assert_eq!(
            comments_map.get(&4),
            Some(&("Docstring of a function", CommentType::Multi))
        );
        assert_eq!(comments_map.get(&6), Some(&("Args:", CommentType::Multi)));
        assert_eq!(
            comments_map.get(&7),
            Some(&("dictarg (dict): A dictionary argument", CommentType::Multi))
        );
        assert_eq!(
            comments_map.get(&19),
            Some(&("Multiline comment that is not a docstring", CommentType::Multi))
        );
        assert_eq!(
            comments_map.get(&26),
            Some(&(
                "Alice's Adventures in Wonderland by Lewis Carroll",
                CommentType::Multi
            ))
        );
        assert_eq!(
            comments_map.get(&27),
            Some(&("A Project Gutenberg eBook", CommentType::Multi))
        );
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

        let comments_collection = CommentCollection::from_comments(comments);
        let json = serde_json::to_string(&comments_collection).unwrap();

        assert!(json.contains("\"single_comments\":{\"1\":\"A class that represents a HttpRequest\"}"));
        assert!(json.contains("\"124\":\"count -> int: The counter of a loop\""));
        assert!(json.contains("\"122\":\"Args:\""));
        assert!(json.contains("\"multiline_comments\""));
    }
}
