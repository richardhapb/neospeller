use super::add_quotes;
use crate::language::{Comment, CommentType, Language};

pub struct Buffer {
    pub lines: Vec<String>,
    pub comments: Vec<Comment>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            comments: Vec::new(),
        }
    }

    pub fn from_string(s: String) -> Self {
        let lines: Vec<String> = s.lines().map(|l| l.to_string()).collect();
        Self {
            lines,
            comments: Vec::new(),
        }
    }

    pub fn push(&mut self, line: String) {
        self.lines.push(line);
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    // Captures comments from a text and returns a JSON object
    pub fn comments_to_json(&self) -> String {
        let mut output = String::new();
        output.push_str("{");

        // Separate comments by type while preserving order
        let (single_comments, ml_comments): (Vec<_>, Vec<_>) = self
            .comments
            .iter()
            .partition(|comment| matches!(comment.comment_type, CommentType::Single));

        // Handle single-line comments
        if !single_comments.is_empty() {
            output.push_str("\"single_comments\": {");
            for comment in &single_comments {
                output.push_str(&format!(
                    "{}: {},",
                    add_quotes(&comment.line.to_string()),
                    add_quotes(&comment.text.clone())
                ));
            }
            output.pop(); // Remove trailing comma
            output.push('}');
        }

        // Handle multi-line comments
        if !ml_comments.is_empty() {
            if !single_comments.is_empty() {
                output.push(',');
            }
            output.push_str("\"multiline_comments\": {");
            for comment in &ml_comments {
                output.push_str(&format!(
                    "{}: {},",
                    add_quotes(&comment.line.to_string()),
                    add_quotes(&comment.text)
                ));
            }
            output.pop(); // Remove trailing comma
            output.push('}');
        }

        output.push_str("}");
        output
    }

    pub fn get_comments(&mut self, language: &Language) -> &Vec<Comment> {
        let mut comments = Vec::new();
        let lines: Vec<&str> = self.lines.iter().map(|l| l.as_str()).collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Skip empty lines
            if line.trim().is_empty() {
                i += 1;
                continue;
            }

            let comment_type = language.get_comment_type(line);

            // Try to parse comment starting at current line
            if let Ok(parse_state) =
                Comment::parse_comment(&language, &self.lines[i..].join("\n"), i + 1, comment_type)
            {
                comments.extend(parse_state.comments);
                if parse_state.lines_parsed > 0 {
                    i += parse_state.lines_parsed;
                } else {
                    i += 1;
                }
            }
        }

        self.comments = comments;
        &self.comments
    }

    pub fn replace_comments(
        &mut self,
        new_comments: &Vec<Comment>,
        language: &Language,
    ) -> Result<(), &'static str> {
        let mut to_add: Vec<&Comment> = vec![]; // Lines added by LLM
        for (i, comment) in new_comments.iter().enumerate() {
            let line = self
                .lines
                .get_mut(comment.line - 1)
                .ok_or("Line not found")?;

            // In case that LLM added a line
            if comment.line > self.comments[i].line {
                to_add.push(&comment);
            }

            let new_line = match comment.comment_type {
                CommentType::Single => {
                    replace_single_comment(line, &self.comments[i].text, &comment.text)
                }
                CommentType::Multi => {
                    replace_multi_comment(line, &self.comments[i].text, &comment.text, &language)
                }
            };

            *line = new_line?;
        }

        // Add the additional lines if LLM adds that
        for &comment in to_add.iter().rev() {
            // TODO: Verify if it is last line of the multiline comment and add symbol_close if it is needed
            let indent = if comment.line > 1 {
                self.lines[comment.line - 1].len() - self.lines[comment.line - 1].len()
            } else {
                0
            };

            let indent_str = " ".repeat(indent);

            let new_line = format!("{}{}", indent_str, &comment.text);
            self.lines.insert(comment.line, new_line);
        }

        Ok(())
    }

    pub fn json_to_comments(
        &mut self,
        json_string: &str,
        language: &Language,
    ) -> Result<&Vec<Comment>, &'static str> {
        let mut comments = vec![];

        comments.extend(serialize_json_element("single_comments", json_string)?);
        comments.extend(serialize_json_element("multiline_comments", json_string)?);

        comments = sort_comments_by_line_number(comments);

        self.replace_comments(&comments, &language)?;

        self.comments = comments;

        Ok(&self.comments)
    }
}

fn serialize_json_element(
    comment_key: &str,
    json_string: &str,
) -> Result<Vec<Comment>, &'static str> {
    // TODO: Refactor this function to be more efficient and use a JSON parser
    let mut comments = vec![];

    let comment_type = match comment_key {
        "single_comments" => CommentType::Single,
        "multiline_comments" => CommentType::Multi,
        _ => return Err("Invalid comment key"),
    };

    if let Some(comments_col) = json_string.find(comment_key) {
        if let Some(open) = json_string[comments_col + 1..].find('{') {
            let open = comments_col + open + 2;
            let mut in_key = true;
            let mut in_value = false;
            let mut in_quotes = false;
            let mut in_scape = false;

            let mut capturing = false;
            let mut inserting = false;
            let mut end = false;
            let mut bracket_count = 1;

            let mut key = String::new();
            let mut value = String::new();

            for c in json_string[open..].chars() {
                if !in_scape {
                    match c {
                        '{' => bracket_count += 1,
                        '"' => in_quotes = !in_quotes,
                        ':' => {
                            if !in_quotes {
                                in_key = false;
                                in_value = true;
                            } else {
                                capturing = true;
                            }
                        }
                        ',' => {
                            if !in_quotes {
                                in_key = true;
                                in_value = false;
                                inserting = true;
                            }
                        }
                        '\\' => in_scape = true,
                        '}' => {
                            if !in_quotes {
                                if bracket_count == 1 {
                                    inserting = true;
                                    end = true;
                                } else {
                                    bracket_count -= 1;
                                }
                            }
                        },
                        ' ' => if in_quotes {
                            capturing = true;
                        },
                        _ => capturing = true,
                    }
                } else {
                    capturing = true;
                }

                if !capturing && !inserting {
                    continue;
                }

                if capturing {
                    if in_key && !in_quotes {
                        return Err("Invalid JSON format");
                    } else if in_key && in_quotes {
                        key.push(c);
                    } else if in_value && in_quotes {
                        value.push(c);
                    }
                }

                if inserting {
                    comments.push(Comment {
                        line: key
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| "Invalid number in key")?,
                        text: value.trim().to_string(),
                        comment_type,
                    });
                    if end {
                        break;
                    }

                    key.clear();
                    value.clear();
                }

                inserting = false;
                capturing = false;
                in_scape = false;
            }
        }
    };
    Ok(comments)
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
fn sort_comments_by_line_number(mut comments: Vec<Comment>) -> Vec<Comment> {
    comments.sort_by_key(|comment| comment.line);
    comments
}

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

        if line[sym_index..]
            .find(&language.ml_comment_symbol_close)
            .is_some()
        {
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
    const RUST_FIXTURE: &str = r#"
        // this is a single line comment
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
        "#;

    const PYTHON_FIXTURE: &str = r#"
        # this is a single line comment
        x = 5

        """
        this is a
        multi-line comment
        """

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

        let mut buffer = Buffer::from_string(RUST_FIXTURE.to_string());
        let comments = buffer.get_comments(&language);

        assert_eq!(comments.len(), 7);

        assert_eq!(comments[0].line, 2);
        assert_eq!(comments[0].text, "this is a single line comment");
        assert_eq!(comments[0].comment_type, CommentType::Single);

        assert_eq!(comments[1].line, 6);
        assert_eq!(comments[1].text, "this is a");
        assert_eq!(comments[1].comment_type, CommentType::Multi);

        assert_eq!(comments[2].text, "multi-line comment");
        assert_eq!(comments[2].comment_type, CommentType::Multi);
        assert_eq!(comments[2].line, 7);

        assert_eq!(
            comments[3].text,
            "Another multi-line comment, but in a single line"
        );
        assert_eq!(comments[3].line, 10);
        assert_eq!(comments[3].comment_type, CommentType::Multi);

        assert_eq!(comments[4].text, "Unestruturated multi-line comment");
        assert_eq!(comments[4].line, 12);
        assert_eq!(comments[4].comment_type, CommentType::Multi);

        assert_eq!(
            comments[5].text,
            "Another unestruturated multi-line comment"
        );
        assert_eq!(comments[5].line, 16);
        assert_eq!(comments[5].comment_type, CommentType::Multi);

        assert_eq!(comments[6].text, "With multiples lines");
        assert_eq!(comments[6].line, 17);
        assert_eq!(comments[6].comment_type, CommentType::Multi);
    }

    #[test]
    fn test_get_comments_python() {
        let language = Language {
            name: "python".to_string(),
            comment_symbol: "#".to_string(),
            ml_comment_symbol: "\"\"\"".to_string(),
            ml_comment_symbol_close: "\"\"\"".to_string(),
        };

        let mut buffer = Buffer::from_string(PYTHON_FIXTURE.to_string());
        let comments = buffer.get_comments(&language);

        println!("{:?}", comments);

        assert_eq!(comments.len(), 13);

        assert_eq!(comments[0].line, 2);
        assert_eq!(comments[0].text, "this is a single line comment");
        assert_eq!(comments[0].comment_type, CommentType::Single);

        assert_eq!(comments[1].line, 6);
        assert_eq!(comments[1].text, "this is a");
        assert_eq!(comments[1].comment_type, CommentType::Multi);

        assert_eq!(comments[2].text, "multi-line comment");
        assert_eq!(comments[2].comment_type, CommentType::Multi);
        assert_eq!(comments[2].line, 7);

        assert_eq!(
            comments[3].text,
            "Another multi-line comment, but in a single line"
        );
        assert_eq!(comments[3].line, 10);
        assert_eq!(comments[3].comment_type, CommentType::Multi);

        assert_eq!(comments[4].text, "Unestruturated multi-line comment");
        assert_eq!(comments[4].line, 12);
        assert_eq!(comments[4].comment_type, CommentType::Multi);

        assert_eq!(
            comments[5].text,
            "Another unestruturated multi-line comment"
        );
        assert_eq!(comments[5].line, 16);
        assert_eq!(comments[5].comment_type, CommentType::Multi);

        assert_eq!(comments[6].text, "With multiples lines");
        assert_eq!(comments[6].line, 17);
        assert_eq!(comments[6].comment_type, CommentType::Multi);

        assert_eq!(comments[7].text, "Print debug information to compare with the visual content in the browser and verify the order.");
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
    fn text_replace_comments() {
        let language = Language {
            name: "python".to_string(),
            comment_symbol: "#".to_string(),
            ml_comment_symbol: "\"\"\"".to_string(),
            ml_comment_symbol_close: "\"\"\"".to_string(),
        };

        let mut buffer = Buffer::from_string(PYTHON_FIXTURE.to_string());
        let comments = buffer.get_comments(&language);

        let mut new_comments = vec![];

        for comment in comments.iter() {
            new_comments.push(Comment {
                line: comment.line,
                text: comment.text.replace('a', "e"),
                comment_type: comment.comment_type,
            })
        }

        buffer.replace_comments(&new_comments, &language).unwrap();

        assert_eq!(buffer.lines[1].trim(), "# this is e single line comment");
        assert_eq!(buffer.lines[4].trim(), "\"\"\"");
        assert_eq!(buffer.lines[5].trim(), "this is e");
        assert_eq!(buffer.lines[6].trim(), "multi-line comment"); // Not change
        assert_eq!(buffer.lines[7].trim(), "\"\"\"");
        assert_eq!(
            buffer.lines[9].trim(),
            "\"\"\"Another multi-line comment, but in e single line\"\"\""
        );
    }

    struct BufferMock {
        comments: Vec<Comment>,
    }

    impl BufferMock {
        fn json_to_comments(
            &mut self,
            json_string: &str,
            _language: &Language,
        ) -> Result<&Vec<Comment>, &'static str> {
            let mut comments = vec![];
            comments.extend(serialize_json_element("single_comments", json_string)?);
            comments.extend(serialize_json_element("multiline_comments", json_string)?);
            self.comments = comments;
            Ok(&self.comments)
        }
    }

    #[test]
    fn text_json_to_comments() {
        let language = Language {
            name: "python".to_string(),
            comment_symbol: "#".to_string(),
            ml_comment_symbol: "\"\"\"".to_string(),
            ml_comment_symbol_close: "\"\"\"".to_string(),
        };

        let mut buffer = BufferMock { comments: vec![] };
        let json_string = r#"{"single_comments": {"1": "A class that represents a HttpRequest"},"multiline_comments": {"122": "Args:","124": "count -> int: The counter of a loop"}}"#;
        let comments = buffer.json_to_comments(json_string, &language).unwrap();

        assert_eq!(comments[0].line, 1);
        assert_eq!(comments[0].text, "A class that represents a HttpRequest");
        assert_eq!(comments[0].comment_type, CommentType::Single);

        assert_eq!(comments[1].line, 122);
        assert_eq!(comments[1].text, "Args:");
        assert_eq!(comments[1].comment_type, CommentType::Multi);

        assert_eq!(comments[2].line, 124);
        assert_eq!(comments[2].text, "count -> int: The counter of a loop");
        assert_eq!(comments[2].comment_type, CommentType::Multi);

        let mut buffer = BufferMock { comments: vec![] };
        let json_string = r#"{  "single_comments": {    "42": "Another comment with spelling mistakes?"  },  "multiline_comments": {    "4": "Docstring of a function",    "6": "Args:",    "7": "dictarg (dict): A dictionary argument",    "19": "Multiline comment that is not a docstring",    "26": "Alice's Adventures in Wonderland by Lewis Carroll",    "27": "A Project Gutenberg eBook"  }}"#;
        let comments = buffer.json_to_comments(json_string, &language).unwrap();

        assert_eq!(comments[0].line, 42);
        assert_eq!(comments[0].text, "Another comment with spelling mistakes?");
        assert_eq!(comments[0].comment_type, CommentType::Single);

        assert_eq!(comments[1].line, 4);
        assert_eq!(comments[1].text, "Docstring of a function");
        assert_eq!(comments[1].comment_type, CommentType::Multi);

        assert_eq!(comments[2].line, 6);
        assert_eq!(comments[2].text, "Args:");
        assert_eq!(comments[2].comment_type, CommentType::Multi);

        assert_eq!(comments[3].line, 7);
        assert_eq!(comments[3].text, "dictarg (dict): A dictionary argument");
        assert_eq!(comments[3].comment_type, CommentType::Multi);

        assert_eq!(comments[4].line, 19);
        assert_eq!(
            comments[4].text,
            "Multiline comment that is not a docstring"
        );
        assert_eq!(comments[4].comment_type, CommentType::Multi);

        assert_eq!(comments[5].line, 26);
        assert_eq!(
            comments[5].text,
            "Alice's Adventures in Wonderland by Lewis Carroll"
        );
        assert_eq!(comments[5].comment_type, CommentType::Multi);

        assert_eq!(comments[6].line, 27);
        assert_eq!(comments[6].text, "A Project Gutenberg eBook");
        assert_eq!(comments[6].comment_type, CommentType::Multi);
    }
}
