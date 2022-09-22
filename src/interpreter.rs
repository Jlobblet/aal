use crate::array::{Array, Atom, GenericArray, IntegerElt, Noun};
use crate::lexer::{lex, Token};
use crate::primitives::{DYADS, MONADS};
use anyhow::Result;
use anyhow::{anyhow, Context};
use std::collections::HashMap;
use std::io::{stdin, stdout, Write};

#[derive(Debug, Clone)]
pub enum Variable {
    Noun(Noun),
    Verb(String),
}

pub fn interpret(mut csl: Vec<Token>, env: &mut HashMap<String, Variable>) -> Result<Option<Noun>> {
    if csl.is_empty() {
        return Ok(None);
    }

    // For now, we will only consider the case where the rightmost value is a numeric noun
    // Safe to unwrap because we know csl is nonempty
    let mut right = get_noun(env, csl.pop().unwrap()).with_context(|| "No rightmost noun")?;

    while let Some(token) = csl.pop() {
        match token {
            Token::Operator(o) => {
                // Look ahead to see if we're in the monadic or dyadic case
                if let Some(Token::Number(_) | Token::Identifier(_)) = csl.last() {
                    match get_noun(env, csl.pop().unwrap()) {
                        Some(n) => {
                            right = DYADS[&o](n, right)?;
                        }
                        None => return Err(anyhow!("Failed to retrieve noun for left operator")),
                    }
                } else {
                    right = MONADS[&o](right)?;
                }
            }
            t => return Err(anyhow!("Nonsensical token {t:?}")),
        }
    }

    Ok(Some(right))
}

fn get_noun(env: &mut HashMap<String, Variable>, tok: Token) -> Option<Noun> {
    match tok {
        Token::Identifier(name) => {
            if let Some(Variable::Noun(n)) = env.get(&name) {
                Some(n.clone())
            } else {
                None
            }
        }
        Token::Number(v) => {
            if v.len() > 1 {
                Some(Noun::Array(Array::Integer(
                    GenericArray::<IntegerElt>::try_from(v.as_slice()).ok()?,
                )))
            } else {
                Some(Noun::Atom(Atom::Integer(v[0].parse().ok()?)))
            }
        }
        _ => None,
    }
}

pub fn repl() -> Result<()> {
    let mut buffer = String::new();
    let mut env = Default::default();
    loop {
        buffer.clear();
        print!("    ");
        stdout().flush()?;
        stdin().read_line(&mut buffer)?;
        match lex(&buffer) {
            Ok(csl) => match interpret(csl, &mut env) {
                Ok(None) => (),
                Ok(Some(n)) => println!("{n:?}"),
                Err(e) => eprintln!("{e}"),
            },
            Err(e) => eprintln!("{e}"),
        }
    }
}
