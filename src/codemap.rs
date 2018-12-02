use std::fmt::Write;
use model::ast::Span;

const TAB_INDENTATION: usize = 4;

pub struct CodeMap<'a> {
    filename: &'a str,
    code: String,
    lines: Vec<String>,  // problem with lifetimes, so we need to have code twice in memory :(
}

impl<'a> CodeMap<'a> {
    pub fn new(filename: &'a str, code: &'a str) -> Self {
        let code = code.replace('\t', &" ".repeat(TAB_INDENTATION));
        let lines = code.split('\n').map(|s| String::from(s)).collect();
        CodeMap {
            filename,
            code,
            lines,
        }
    }

    pub fn get_code(&self) -> &str {
        &self.code
    }

    pub fn format_message(&self, span: Span, msg: &str) -> String {
        assert!(span.0 <= span.1);
        let mut result = String::new();

        let beg_row_col = self.find_row_col(span.0);
        let end_row_col = self.find_row_col(span.1);

        match beg_row_col {
            Some((row, col)) => {
                write!(&mut result, "{}:{}:{}:\n", self.filename, row, col);
            },
            None => {
                write!(&mut result, "{}:{}:\n", self.filename, span.0);
            },
        };

        if let (Some((row0, col0)), Some((row1, col1))) = (beg_row_col, end_row_col) {
            if row0 == row1 {
                write!(&mut result, "{}\n", self.lines[row0]);
                write!(&mut result, "{}{}\n", " ".repeat(col0), "^".repeat(col1 - col0));
            }
            else {
                write!(&mut result, "/-{}v\n", "-".repeat(col0));
                for i in row0..=row1 {
                    write!(&mut result, "| {}\n", self.lines[i]);
                }
                write!(&mut result, "\\-{}^\n", "-".repeat(col1));
            }
        }

        write!(&mut result, "{}\n\n", msg);

        result
    }

    fn find_row_col(&self, pos: usize) -> Option<(usize, usize)> {
        let mut cur_pos = 0usize;
        let mut row = 0;

        for line in &self.lines {
            if pos <= cur_pos + line.len() + 1 {
                return Some((row, pos - cur_pos));
            }
            cur_pos += line.len() + 1;
            row += 1;
        }

        None
    }
}
