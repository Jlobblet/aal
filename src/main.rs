use anyhow::Result;

mod array;
mod interpreter;
mod lexer;
mod primitives;

fn main() -> Result<()> {
    interpreter::repl()
}
