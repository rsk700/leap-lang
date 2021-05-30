use std::fs;

#[derive(Debug)]
pub struct PathError {
    pub error: String,
    pub path: String,
    pub position: usize,
}

impl PathError {
    pub fn new(error: String, path: String, position: usize) -> Self {
        Self {
            error,
            path,
            position,
        }
    }

    pub fn error_report(&self) -> String {
        // file path, line number, position in line
        let error_line = self.get_error_line();
        match error_line {
            Some((l, p, line)) => {
                let arrow_padding = String::from_utf8(vec![b' '; p]).unwrap();
                format!(
                    "{}:{}:{}\n     |\n{:>4} |{}\n     |{}^---\n\n{}",
                    self.path, l, p, l, line, arrow_padding, self.error
                )
            }
            None => format!("{}\n{}", self.path, self.error),
        }
    }

    fn get_error_line(&self) -> Option<(usize, usize, String)> {
        if let Ok(s) = fs::read_to_string(&self.path) {
            let mut position: usize = 0;

            for (n, line) in s.split('\n').enumerate() {
                if self.position <= position + line.len() + 1 {
                    return Some((n, self.position - position, line.to_owned()));
                }
                position += line.len() + 1;
            }
        }
        None
    }
}
