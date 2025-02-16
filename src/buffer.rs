pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn from_string(s: String) -> Self {
        let lines: Vec<String> = s.lines().map(|l| l.to_string()).collect();
        Self { lines }
    }

    pub fn push(&mut self, line: String) {
        self.lines.push(line);
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }
}

