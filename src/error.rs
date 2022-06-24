// Querybinder -- Generate boilerplate from SQL for statically typed languages
// Copyright 2022 Ruud van Asseldonk

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

use std::path::Path;

use crate::Span;

pub trait Error {
    /// The source location of the error.
    fn span(&self) -> Span;

    /// The error message.
    ///
    ///  * Shorter is better.
    ///  * Simpler is better (no jargon).
    ///  * The expected thing goes first, the actual thing goes second.
    fn message(&self) -> &str;

    /// Optionally, a note about error.
    ///
    /// For example, an unmatched parenthesis can point to the opening paren.
    fn note(&self) -> Option<(&str, Option<Span>)>;

    /// Optionally, a hint on how to fix the problem.
    fn hint(&self) -> Option<&str>;
}

impl dyn Error {
    pub fn print(&self, fname: &Path, input: &[u8]) {
        let highlight = highlight_span_in_line(fname, input, self.span());
        print!("Error: {}\n{}", self.message(), highlight);

        if let Some((note, opt_note_span)) = self.note() {
            println!("Note: {}", note);
            if let Some(note_span) = opt_note_span {
                let highlight = highlight_span_in_line(fname, input, note_span);
                print!("{}", highlight);
            }
        }

        if let Some(hint) = self.hint() {
            println!("Hint: {}", hint);
        }
    }
}

fn highlight_span_in_line(fname: &Path, input: &[u8], span: Span) -> String {
    use std::cmp;
    use std::iter;
    use std::fmt::Write;

    // Locate the line that contains the error.
    let mut line = 1;
    let mut line_start = 0;
    let mut line_end = 0;
    for (&c, i) in input.iter().zip(0..) {
        if i == span.start { break }
        if c == b'\n' {
            line += 1;
            line_start = i + 1;
        }
    }
    for (&c, i) in input[line_start..].iter().zip(line_start..) {
        if c == b'\n' {
            line_end = i;
            break
        }
    }

    // Try as best as we can to report the error. However, if the parse failed
    // because the input was invalid UTF-8, there is little we can do.
    let line_content = String::from_utf8_lossy(&input[line_start..line_end]);

    // The length of the mark can be longer than the line, for example when
    // token to mark was a multiline string literal. In that case, highlight
    // only up to the newline, don't extend the tildes too far.
    let mark_len = cmp::max(
        1,
        cmp::min(span.len(), line_content.len() + line_start - span.start),
    );

    let line_num_str = line.to_string();
    let line_num_pad: String = line_num_str.chars().map(|_| ' ').collect();
    // TODO: Use unicode-width to determine this, don't just count the bytes.
    let mark_indent: String = iter::repeat(' ').take(span.start - line_start).collect();
    let mark_under: String = iter::repeat('~').take(mark_len).collect();
    let fname_str = fname.to_string_lossy();

    let mut result = String::new();
    // Note, the unwraps here are safe because writing to a string does not fail.
    writeln!(&mut result, "--> {}:{}:{}", fname_str, line, span.start - line_start).unwrap();
    writeln!(&mut result, "{} |", line_num_pad).unwrap();
    writeln!(&mut result, "{} | {}", line_num_str, line_content).unwrap();
    writeln!(&mut result, "{} | {}^{}", line_num_pad, mark_indent, &mark_under[1..]).unwrap();

    result
}

#[derive(Debug)]
pub struct ParseError {
    pub span: Span,
    pub message: &'static str,
}

impl From<ParseError> for Box<dyn Error> {
    fn from(err: ParseError) -> Self {
        Box::new(err)
    }
}

impl Error for ParseError {
    fn span(&self) -> Span { self.span }
    fn message(&self) -> &str { self.message }
    fn note(&self) -> Option<(&str, Option<Span>)> { None }
    fn hint(&self) -> Option<&str> { None }
}

/// A parse result, either the parsed value, or a parse error.
pub type PResult<T> = std::result::Result<T, ParseError>;
