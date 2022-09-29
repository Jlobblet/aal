#![forbid(unsafe_code)]
#![deny(missing_copy_implementations, missing_debug_implementations)]

use anyhow::Result;

mod arrays;
mod interpreter;
mod lexer;
mod primitives;

fn main() -> Result<()> {
    interpreter::repl()
}
