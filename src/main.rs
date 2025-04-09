mod error;
use std::fs;

pub use error::{Error, Result};

const MACRO: &str = "html!";
const OPEN: char = '{';
const CLOSE: char = '}';
const INDENT: usize = 4;

fn main() -> Result<()> {
    let mut f = fs::read_to_string("src/main.rs")?
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    fix_indent(&mut f);

    // println!("{:?}", f);
    // f.iter().for_each(|s| println!("{}", get_indent_level(s)));

    fs::write("src/main.rs", f.join("\n"))?;

    Ok(())
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Span {
    start: usize,
    end: usize,
}

impl Span {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

fn get_indent_level(line: &str) -> usize {
    let mut tabs = 0;
    let mut spaces = 0;

    for c in line.chars() {
        match c {
            '\t' => tabs += 1,
            ' ' => spaces += 1,
            _ => break,
        }
    }
    assert!(tabs <= 1);
    spaces
}

fn fix_indent(buf: &mut Vec<String>) {
    let spans = find_macro(&buf);
    for span in spans {
        let mut current_indent = get_indent_level(&buf[span.start]);
        for i in (span.start)..=span.end {
            let line = &mut buf[i];
            *line = line.trim_start().to_string();

            if balance(line) == 1 {
                *line = format!("{}{}", " ".repeat(current_indent), line);
                current_indent += INDENT;
            } else if line.starts_with(CLOSE) {
                current_indent = current_indent.saturating_sub(INDENT);

                *line = format!("{}{}", " ".repeat(current_indent), line);
            } else {
                *line = format!("{}{}", " ".repeat(current_indent), line);
            }
        }
    }
}

fn balance(str: &str) -> i32 {
    let mut open_count = 0;
    let mut close_count = 0;
    let mut inside_string = false;

    for c in str.chars() {
        // TODO: escaped quotes
        if c == '"' {
            inside_string = !inside_string;
        }

        if c == OPEN && !inside_string {
            open_count += 1;
        } else if c == CLOSE && !inside_string {
            close_count += 1;
        }
    }

    if open_count > close_count {
        return 1;
    } else if close_count > open_count {
        return -1;
    } else {
        return 0;
    }
}

fn find_macro(buf: &[String]) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut brackets = 0i32;
    let mut inside_macro = false;
    let mut span = Span::new(0, 0);

    for (i, line) in buf.iter().enumerate() {
        if line.starts_with("//") {
            continue;
        }
        if line.contains(MACRO) {
            inside_macro = true;
            span.start = i;
        }
        if inside_macro {
            if line.contains(OPEN) {
                brackets += 1;
            } else if line.contains(CLOSE) {
                brackets -= 1;
            }
            if brackets == 0 {
                span.end = i;
                if span.start < span.end {
                    spans.push(span);
                }
                inside_macro = false;
            }
        }
    }
    spans
}

struct FileBuffer {
    buf: Vec<String>,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_find_span() {
        let buf = vec![
            "hello world".to_string(),
            "html! {".to_string(),
            "nonsense".to_string(),
            "}".to_string(),
        ];
        let spans = find_macro(&buf);
        assert_eq!(spans, vec![Span::new(1, 3)]);
    }

    #[test]
    fn test_fix_indent() {
        let mut buf = vec![
            "hello world".to_string(),
            "html! {".to_string(),
            "nonsense".to_string(),
            "}".to_string(),
        ];

        let expected = vec![
            "hello world".to_string(),
            "html! {".to_string(),
            "    nonsense".to_string(),
            "}".to_string(),
        ];
        let res = fix_indent(&mut buf);
        println!("{:?}", res);
        assert_eq!(buf, expected);
    }

    #[test]
    fn fix_indent_complex() {
        let mut buf = vec![
            "async fn handler() -> impl IntoResponse {".to_string(),
            "    html::fullpage(html! {".to_string(),
            "        h1 { \"Hello from rs\" }".to_string(),
            "            })".to_string(),
            "           }".to_string(),
        ];

        let expected = vec![
            "async fn handler() -> impl IntoResponse {".to_string(),
            "    html::fullpage(html! {".to_string(),
            "        h1 { \"Hello from rs\" }".to_string(),
            "    })".to_string(),
            "}".to_string(),
        ];

        fix_indent(&mut buf);
        assert_eq!(buf, expected);
    }
}
