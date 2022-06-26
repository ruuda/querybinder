#![no_main]

use libfuzzer_sys::fuzz_target;
use querybinder::lexer::sql::Lexer;
use querybinder::parser::document::Parser;

fuzz_target!(|input: &str| {
    // Parsing may fail, but it should not hang or panic.
    let lexer = Lexer::new(&input);
    let tokens = lexer.run();
    let mut parser = Parser::new(&input, &tokens);
    let _ = parser.parse_document();
});
