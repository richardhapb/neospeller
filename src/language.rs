
pub enum CommentType {
    Single,
    Multi,
}

pub struct Comment<'a> {
    pub line: usize,
    pub text: String,
    pub comment_type: &'a CommentType,
}

pub struct Language {
    pub name: String,
    pub comment_symbol: String,
    pub ml_comment_symbol: String,
    pub ml_comment_symbol_close: String,
}

impl Language {
    pub fn get_comment_type(&self, line: &str) -> &CommentType {
        if line.find(&self.ml_comment_symbol).is_some() {
            return &CommentType::Multi;
        }

        &CommentType::Single
    }

    pub fn get_comments(&self, input: &str) -> Vec<Comment> {
        let mut comments = Vec::new();
        let symbol_close = &self.ml_comment_symbol_close;
        let special_chars = vec!['*', '-', '+', '='];

        let lines = input.lines();
        let mut capturing = false;

        for (i, line) in lines.enumerate() {
            let comment_type = if capturing {
                &CommentType::Multi
            } else {
                self.get_comment_type(&line)
            };

            let trimed_line = line.trim();
            let closure = trimed_line.find(symbol_close);

            let symbol = match self.get_comment_type(line) {
                CommentType::Single => &self.comment_symbol,
                CommentType::Multi => &self.ml_comment_symbol,
            };

            if capturing && closure.is_some() {
                capturing = false;

                let closure = closure.unwrap();

                if closure + trimed_line.len() > symbol_close.len() {
                    comments.push(Comment {
                        line: i,
                        text: trimed_line[..closure].trim().to_string(),
                        comment_type,
                    })
                }
                continue;
            }

            if let Some(col) = trimed_line.find(symbol) {
                let comment_length = trimed_line.len();
                let cut = match comment_type {
                    CommentType::Multi => {
                        if comment_length > symbol.len() {
                            capturing = false;
                            let mut cut = 0;
                            for c in special_chars.iter() {
                                if trimed_line[col + symbol.len()..].replace(*c, "").len() == 0 {
                                    capturing = true;
                                    cut = col + symbol.len();
                                    break;
                                }
                            }

                            if cut == 0 {
                                if symbol == symbol_close {
                                    cut = trimed_line.replace(symbol, "").len() + symbol.len();
                                } else {
                                    cut = trimed_line.replace(symbol_close, "").len();
                                }
                            }

                            Some(cut)
                        } else {
                            capturing = true;
                            Some(comment_length)
                        }
                    }

                    CommentType::Single => Some(comment_length),
                };

                let cut = cut.unwrap_or(comment_length);

                let text = trimed_line[col + symbol.len()..cut].trim().to_string();

                if text.len() > 0 {
                    comments.push(Comment {
                        line: i + 1,
                        text,
                        comment_type,
                    });
                }
                continue;
            }

            if capturing {
                comments.push(Comment {
                    line: i + 1,
                    text: trimed_line.to_string(),
                    comment_type,
                });
            }
        }

        comments
    }
}
