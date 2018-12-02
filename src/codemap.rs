use std::fmt::Write;
use model::ast::Span;

pub fn format_message(filename: &str, span: Span, code: &str, msg: &str) -> String {
    assert!(span.0 <= span.1);
    let mut result = String::new();

    let lines: Vec<_> = code.split('\n').collect();
    let beg_row_col = find_row_col(&lines, span.0);
    let end_row_col = find_row_col(&lines, span.1);

    match beg_row_col {
        Some((row, col)) => {
            write!(&mut result, "{}:{}:{}:\n", filename, row, col);
        },
        None => {
            write!(&mut result, "{}:{}:\n", filename, span.0);
        },
    };

    if let (Some((row0, col0)), Some((row1, col1))) = (beg_row_col, end_row_col) {
        if row0 == row1 {
            write!(&mut result, "{}\n", lines[row0]);
            write!(&mut result, "{}{}\n", " ".repeat(col0), "^".repeat(col1 - col0));
        }
        else {
            write!(&mut result, "/-{}v\n", "-".repeat(col0));
            for i in row0..=row1 {
                write!(&mut result, "| {}\n", lines[i]);
            }
            write!(&mut result, "\\-{}^\n", "-".repeat(col1));
        }
    }

    write!(&mut result, "{}\n\n", msg);

    result
}

fn find_row_col(lines: &Vec<&str>, pos: usize) -> Option<(usize, usize)> {
    let mut cur_pos = 0usize;
    let mut row = 0;

    for line in lines {
        if pos <= cur_pos + line.len() + 1 {
            return Some((row, pos - cur_pos));
        }
        cur_pos += line.len() + 1;
        row += 1;
    }

    None
}
