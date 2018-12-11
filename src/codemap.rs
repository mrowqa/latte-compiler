use colored::*;
use model::ast::Span;
use std::fmt::Write;

const TAB_INDENTATION: usize = 4;
const ERROR_CONTEXT_LINES_MARGIN: usize = 2;

pub struct CodeMap<'a> {
    filename: &'a str,
    code: String,
    lines: Vec<String>, // problem with lifetimes, so we need to have code twice in memory :(
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
        let err_fmt = |s: &str| s.red().bold();

        // empty span means just a message, without localisation
        if span.0 != span.1 {
            let beg_row_col = self.find_row_col(span.0);
            let end_row_col = self.find_row_col(span.1);

            match beg_row_col {
                Some((row, col)) => {
                    writeln!(&mut result, "{}:{}:{}:", self.filename, row, col).unwrap();
                }
                None => {
                    writeln!(&mut result, "{}:{}:", self.filename, span.0).unwrap();
                }
            };

            if let (Some((row0, col0)), Some((row1, col1))) = (beg_row_col, end_row_col) {
                let indent = if row0 == row1 { "" } else { "  " };
                let lo_ind = if row0 < ERROR_CONTEXT_LINES_MARGIN {
                    0
                } else {
                    row0 - ERROR_CONTEXT_LINES_MARGIN
                };
                for i in lo_ind..row0 {
                    writeln!(&mut result, "{}{}", indent, self.lines[i]).unwrap();
                }

                if row0 == row1 {
                    writeln!(&mut result, "{}", self.lines[row0]).unwrap();
                    writeln!(
                        &mut result,
                        "{}{}",
                        " ".repeat(col0),
                        err_fmt(&"^".repeat(col1 - col0))
                    )
                    .unwrap();
                } else {
                    writeln!(
                        &mut result,
                        "{}{}{}",
                        err_fmt("/-"),
                        err_fmt(&"-".repeat(col0)),
                        err_fmt("v")
                    )
                    .unwrap();
                    for i in row0..=row1 {
                        writeln!(&mut result, "{} {}", err_fmt("|"), self.lines[i]).unwrap();
                    }
                    writeln!(
                        &mut result,
                        "{}{}{}",
                        err_fmt("\\"),
                        err_fmt(&"-".repeat(col1)),
                        err_fmt("^")
                    )
                    .unwrap();
                }

                for i in (row1 + 1)..(row1 + 1 + ERROR_CONTEXT_LINES_MARGIN) {
                    if i >= self.lines.len() {
                        break;
                    }
                    writeln!(&mut result, "{}{}", indent, self.lines[i]).unwrap();
                }
            }
        }

        write!(&mut result, "{}\n\n", err_fmt(msg)).unwrap();

        result
    }

    fn find_row_col(&self, pos: usize) -> Option<(usize, usize)> {
        let mut cur_pos = 0usize;

        for (row, line) in self.lines.iter().enumerate() {
            if pos < cur_pos + line.len() + 1 {
                return Some((row, pos - cur_pos));
            }
            cur_pos += line.len() + 1;
        }

        None
    }
}
