use std::fmt::Write;
use std::cmp::max;
use model::ast::Span;
use colored::*;

const TAB_INDENTATION: usize = 4;
const ERROR_CONTEXT_LINES_MARGIN: usize = 2;

pub struct CodeMap<'a> {
    filename: &'a str,
    code: String,
    lines: Vec<String>,  // problem with lifetimes, so we need to have code twice in memory :(
}

impl<'a> CodeMap<'a> {
    pub fn new(filename: &'a str, code: &'a str) -> Self {
        let code = code.replace('\t', &" ".repeat(TAB_INDENTATION));
        let lines = code.split('\n').map(String::from).collect();
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

        let err_fmt = |s: &str| s.red().bold();

        match beg_row_col {
            Some((row, col)) => {
                write!(&mut result, "{}:{}:{}:\n", self.filename, row, col);
            },
            None => {
                write!(&mut result, "{}:{}:\n", self.filename, span.0);
            },
        };

        if let (Some((row0, col0)), Some((row1, col1))) = (beg_row_col, end_row_col) {
            let indent = if row0 == row1 {""} else {"  "};
            for i in max(0, row0 - ERROR_CONTEXT_LINES_MARGIN)..row0 {
                write!(&mut result, "{}{}\n", indent, self.lines[i]);
            }

            if row0 == row1 {
                write!(&mut result, "{}\n", self.lines[row0]);
                write!(&mut result, "{}{}\n",
                    " ".repeat(col0), err_fmt(&"^".repeat(col1 - col0)));
            }
            else {
                write!(&mut result, "{}{}{}\n",
                    err_fmt("/-"), err_fmt(&"-".repeat(col0)), err_fmt("v"));
                for i in row0..=row1 {
                    write!(&mut result, "{} {}\n", err_fmt("|"), self.lines[i]);
                }
                write!(&mut result, "{}{}{}\n",
                    err_fmt("\\-"), err_fmt(&"-".repeat(col1 - 1)), err_fmt("^"));
            }

            for i in (row1 + 1)..(row1 + 1 + ERROR_CONTEXT_LINES_MARGIN) {
                if i >= self.lines.len() {
                    break;
                }
                write!(&mut result, "{}{}\n", indent, self.lines[i]);
            }
        }

        write!(&mut result, "{}\n\n", err_fmt(msg));

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
