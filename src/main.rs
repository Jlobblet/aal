use anyhow::{Result};

mod lexer;
mod primitives;
mod interpreter;
mod array;

fn main() -> Result<()> {
    interpreter::repl()
}
