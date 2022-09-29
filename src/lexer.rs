use anyhow::Context;

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

impl fsm_lexer::InputClass for InputClass {
    fn classify(c: char) -> Self {
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

impl fsm_lexer::StateTransitionTable<InputClass> for LexerState {
    fn transition(self, class: Option<InputClass>) -> (Self, fsm_lexer::LexerAction) {
        use InputClass::*;
        use fsm_lexer::LexerAction::*;
        use LexerState as LS;
        use LexerState::*;
        use InputClass as IC;

        match self {
            Initial | LS::Whitespace => match class {
                Some(IC::Other | Dot | Colon) => (LS::Other, Advance),
                Some(IC::Whitespace) => (LS::Whitespace, NoAction),
                Some(Letter) => (Alphanum, Advance),
                Some(Digit) => (Num, Advance),
                Some(IC::Quote) => (LS::Quote, Advance),
                None | Some(IC::LF) => (LS::LF, Advance),
            },

            LS::Other => match class {
                Some(IC::Other) => (LS::Other, EmitAndAdvance),
                Some(IC::Whitespace) => (LS::Whitespace, EmitAndReset),
                Some(Letter) => (Alphanum, EmitAndAdvance),
                Some(Digit) => (Num, EmitAndAdvance),
                Some(Dot | Colon) => (LS::Other, NoAction),
                Some(IC::Quote) => (LS::Quote, EmitAndAdvance),
                None | Some(IC::LF) => (LS::LF, EmitAndAdvance),
            },

            Alphanum => match class {
                Some(IC::Other) => (LS::Other, EmitAndAdvance),
                Some(IC::Whitespace) => (LS::Whitespace, EmitAndReset),
                Some(Letter | Digit) => (Alphanum, NoAction),
                Some(Dot | Colon) => (LS::Other, NoAction),
                Some(IC::Quote) => (LS::Quote, EmitAndAdvance),
                None | Some(IC::LF) => (LS::LF, EmitAndAdvance),
            },

            Num => match class {
                Some(IC::Other) => (LS::Other, AppendAndAdvance),
                Some(IC::Whitespace) => (LS::Whitespace, AppendAndReset),
                Some(Letter | Digit | Dot) => (Num, NoAction),
                Some(Colon) => (LS::Other, NoAction),
                Some(IC::Quote) => (LS::Quote, AppendAndAdvance),
                None | Some(IC::LF) => (LS::LF, AppendAndAdvance),
            },

            LS::Quote => match class {
                Some(IC::Quote) => (DoubleQuote, NoAction),
                None => (LS::LF, Stop),
                _ => (LS::Quote, NoAction),
            },

            DoubleQuote => match class {
                Some(IC::Other | Dot | Colon) => (LS::Other, EmitAndAdvance),
                Some(IC::Whitespace) => (LS::Whitespace, EmitAndReset),
                Some(Letter) => (Alphanum, EmitAndAdvance),
                Some(Digit) => (Num, EmitAndAdvance),
                Some(IC::Quote) => (LS::Quote, NoAction),
                None | Some(IC::LF) => (LS::LF, EmitAndAdvance),
            },

            LS::LF => match class {
                Some(IC::Other | Dot | Colon) => (LS::Other, EmitAndAdvance),
                Some(IC::Whitespace) => (LS::Whitespace, EmitAndAdvance),
                Some(Letter) => (Alphanum, EmitAndAdvance),
                Some(Digit) => (Num, EmitAndAdvance),
                Some(IC::Quote) => (LS::Quote, EmitAndAdvance),
                None | Some(IC::LF) => (LS::LF, EmitAndAdvance),
            },
        }
    }
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
    Eol,
}

impl fsm_lexer::Token<LexerState> for Token {
    fn emit(s: String, state: LexerState) -> Self {
        use LexerState::*;
        use Token::*;
        let token_fn = match state {
            Other => Operator,
            Alphanum => Identifier,
            Num => |s| Number(vec![s]),
            Quote | DoubleQuote => StringLiteral,
            LF => |_| Eol,
            _ => unreachable!("Attempted to create a token from nonsensical state"),
        };
        token_fn(s)
    }

    fn append(s: String, state: LexerState, last: Option<&mut Self>) -> Option<Self> {
        use LexerState::*;
        use Token::*;
        match (state, last) {
            (Num, Some(Number(v))) => { v.push(s); None },
            _ => Some(Self::emit(s, state)),

        }
    }
}

pub fn lex(input: &str) -> anyhow::Result<Vec<Token>> {
    let lexer = fsm_lexer::Lexer::new(LexerState::Initial);
    lexer.lex(input).context("Failed to lex")
}