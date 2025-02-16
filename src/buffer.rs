use super::add_quotes;
use crate::language::{Comment, CommentType, Language};
use std::collections::BTreeMap;

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
        let mut single_comments: BTreeMap<String, String> = Default::default();
        let mut ml_comments: BTreeMap<String, String> = Default::default();

        output.push_str("{");

        for comment in self.comments.iter() {
            match comment.comment_type {
                CommentType::Single => single_comments.insert(
                    add_quotes(&comment.line.to_string()),
                    add_quotes(&comment.text.clone()),
                ),
                CommentType::Multi => ml_comments.insert(
                    add_quotes(&comment.line.to_string()),
                    add_quotes(&comment.text),
                ),
            };
        }

        if single_comments.len() > 0 {
            output.push_str("\"single_comments\": {");
            for (lineno, text) in &single_comments {
                output.push_str(&format!("{}: {},", lineno, text));
            }
            output.pop();
            output.push('}');
        }

        if ml_comments.len() > 0 {
            if single_comments.len() > 0 {
                output.push(',');
            }

            output.push_str("\"multiline_comments\": {");
            for (lineno, text) in &ml_comments {
                output.push_str(&format!("{}: {},", lineno, text));
            }
            output.pop();
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
                i += parse_state.lines_parsed + 1;
            } else {
                i += 1;
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

        comments.extend(parse_json_element("single_comments", json_string)?);
        comments.extend(parse_json_element("multiline_comments", json_string)?);

        self.replace_comments(&comments, &language)?;

        self.comments = comments;

        Ok(&self.comments)
    }
}

fn parse_json_element(comment_key: &str, json_string: &str) -> Result<Vec<Comment>, &'static str> {
    let mut comments = vec![];

    if let Some(single_comments_col) = json_string.find(comment_key) {
        if let Some(open) = json_string[single_comments_col + 1..].find('{') {
            let open = single_comments_col + open + 2;
            if let Some(close) = json_string[open..].find('}') {
                let close = open + close;
                let items = json_string[open..close].split(',');
                for item in items {
                    let (key, value) = item.split_once(':').ok_or("Invalid JSON format")?;

                    let key = key.replace("\"", "");
                    let value = value.replace("\"", "");

                    let comment_type = match comment_key {
                        "single_comments" => CommentType::Single,
                        "multiline_comments" => CommentType::Multi,
                        _ => return Err("Invalid comment key"),
                    };

                    comments.push(Comment {
                        line: key
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| "Invalid number in key")?,
                        text: value.trim().to_string(),
                        comment_type,
                    })
                }
            }
        };
    };
    Ok(comments)
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
            comments.extend(parse_json_element("single_comments", json_string)?);
            comments.extend(parse_json_element("multiline_comments", json_string)?);
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

        let mut buffer = BufferMock {comments: vec![]};
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
    }
}
