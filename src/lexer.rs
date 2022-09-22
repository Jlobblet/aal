use anyhow::anyhow;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum InputClass {
    Other,
    Whitespace,
    Letter,
    Digit,
    Dot,
    Colon,
    Quote,
    LF,
}

impl InputClass {
    pub fn classify(c: char) -> Self {
        use InputClass::*;
        match c {
            ' ' | '\t' => Whitespace,
            'a'..='z' | 'A'..='Z' => Letter,
            '0'..='9' | '_' => Digit,
            '.' => Dot,
            ':' => Colon,
            '"' => Quote,
            '\n' => LF,
            _ => Other,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum LexerState {
    Initial,
    Whitespace,
    Other,
    Alphanum,
    Num,
    Quote,
    DoubleQuote,
    LF,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum LexerAction {
    NoAction,
    Advance,
    EmitAndAdvance,
    EmitAndReset,
    AppendAndAdvance,
    AppendAndReset,
    Stop,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(Vec<String>),
    Operator(String),
    StringLiteral(String),
}

pub fn lex(input: &str) -> anyhow::Result<Vec<Token>> {
    use anyhow::Context;
    use InputClass as IC;
    use InputClass::*;
    use LexerAction::*;
    use LexerState as LS;
    use LexerState::*;

    let input = input.chars().collect::<Vec<_>>();

    let mut current_index = 0;
    let mut word_index = Some(0);
    let mut current_state = Initial;

    let mut output = Vec::new();
    while current_index < input.len() {
        let class = InputClass::classify(input[current_index]);
        let (next_state, action) = match current_state {
            Initial | LS::Whitespace => match class {
                IC::Other | Dot | Colon => (LS::Other, Advance),
                IC::Whitespace => (LS::Whitespace, NoAction),
                Letter => (Alphanum, Advance),
                Digit => (Num, Advance),
                IC::Quote => (LS::Quote, Advance),
                IC::LF => (LS::LF, Advance),
            },

            LS::Other => match class {
                IC::Other => (LS::Other, EmitAndAdvance),
                IC::Whitespace => (LS::Whitespace, EmitAndReset),
                Letter => (Alphanum, EmitAndAdvance),
                Digit => (Num, EmitAndAdvance),
                Dot | Colon => (LS::Other, NoAction),
                IC::Quote => (LS::Quote, EmitAndAdvance),
                IC::LF => (LS::LF, EmitAndAdvance),
            },

            Alphanum => match class {
                IC::Other => (LS::Other, EmitAndAdvance),
                IC::Whitespace => (LS::Whitespace, EmitAndReset),
                Letter | Digit => (Alphanum, NoAction),
                Dot | Colon => (LS::Other, NoAction),
                IC::Quote => (LS::Quote, EmitAndAdvance),
                IC::LF => (LS::LF, EmitAndAdvance),
            },

            Num => match class {
                IC::Other => (LS::Other, AppendAndAdvance),
                IC::Whitespace => (LS::Whitespace, AppendAndReset),
                Letter | Digit | Dot => (Num, NoAction),
                Colon => (LS::Other, NoAction),
                IC::Quote => (LS::Quote, AppendAndAdvance),
                IC::LF => (LS::LF, AppendAndAdvance),
            },

            LS::Quote => match class {
                IC::Quote => (DoubleQuote, NoAction),
                _ => (LS::Quote, NoAction),
            },

            DoubleQuote => match class {
                IC::Other | Dot | Colon => (LS::Other, EmitAndAdvance),
                IC::Whitespace => (LS::Whitespace, EmitAndReset),
                Letter => (Alphanum, EmitAndAdvance),
                Digit => (Num, EmitAndAdvance),
                IC::Quote => (LS::Quote, NoAction),
                IC::LF => (LS::LF, EmitAndAdvance),
            },

            LS::LF => match class {
                IC::Other | Dot | Colon => (LS::Other, EmitAndAdvance),
                IC::Whitespace => (LS::Whitespace, EmitAndAdvance),
                Letter => (Alphanum, EmitAndAdvance),
                Digit => (Num, EmitAndAdvance),
                IC::Quote => (LS::Quote, EmitAndAdvance),
                IC::LF => (LS::LF, EmitAndAdvance),
            },
        };

        if action == Stop {
            break;
        }

        // Emit words
        match action {
            EmitAndAdvance | EmitAndReset | AppendAndAdvance | AppendAndReset => {
                let word_index = word_index
                    .with_context(|| anyhow!("should have a word_index when emitting: {current_index}, {current_state:?}, {class:?} => {next_state:?}, {action:?}"))?;
                let text = input[word_index..current_index].iter().collect::<String>();

                match output.last_mut() {
                    Some(Token::Number(v))
                        if action == AppendAndAdvance || action == AppendAndReset =>
                    {
                        v.push(text)
                    }
                    _ => {
                        let token_type = match current_state {
                            LS::Other => Token::Operator,
                            Alphanum => Token::Identifier,
                            Num => |s| Token::Number(vec![s]),
                            LS::Quote | DoubleQuote => Token::StringLiteral,
                            _ => return Err(anyhow!("attempted to emit a token with no corresponding token type: {current_index}, {current_state:?}, {class:?} => {next_state:?}, {action:?}")),
                        };
                        output.push(token_type(text));
                    }
                }
            }
            _ => (),
        }

        // Update word index
        match action {
            Advance | EmitAndAdvance | AppendAndAdvance => word_index = Some(current_index),
            EmitAndReset | AppendAndReset => word_index = None,
            _ => (),
        }

        current_state = next_state;
        current_index += 1;
    }
    Ok(output)
}
