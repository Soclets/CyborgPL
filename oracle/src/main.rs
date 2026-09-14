//! `oracle file.cpls` — run a CyborgPL program the slow, obvious way.
//!
//! This is the reference interpreter: the executable specification. It knows
//! exactly what `design/` has decided and refuses everything else with a
//! message that says so. Where the design leaves a question open and the
//! interpreter has to pick, the pick is marked "provisional" in the source
//! and listed in oracle/README.md.

#![allow(dead_code)]

mod ast;
mod interp;
mod lexer;
mod parser;
mod value;

use std::process::exit;

pub fn run_source(src: &str, out: Box<dyn std::io::Write>) -> Result<u8, String> {
    let toks = lexer::lex(src).map_err(|e| format!("line {}:{}: {}", e.line, e.col, e.msg))?;
    let prog = parser::Parser::new(toks).program().map_err(|e| if e.col == 0 { format!("line {}: {}", e.line, e.msg) } else { format!("line {}:{}: {}", e.line, e.col, e.msg) })?;
    let mut interp = interp::Interp::new(&prog, out);
    interp.run().map_err(|e| if e.line == 0 { e.msg } else { format!("line {}: {}", e.line, e.msg) })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: oracle <file.cpls>");
        exit(2);
    }
    let path = &args[1];
    if !path.ends_with(".cpls") {
        eprintln!("oracle: a CyborgPL source file ends in .cpls");
        exit(2);
    }
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("oracle: cannot read {path}: {e}"); exit(2); }
    };
    match run_source(&src, Box::new(std::io::stdout())) {
        Ok(code) => exit(code as i32),
        Err(msg) => { eprintln!("{path}: {msg}"); exit(1); }
    }
}
