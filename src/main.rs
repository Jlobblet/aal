use std::cmp::Ordering;
use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use num_traits::{abs, Signed};
use phf::phf_map;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{stdin, stdout, Write};
use std::iter::zip;
use std::ops::{Add, Sub};
use std::str::FromStr;

type IntegerElt = i32;
type DecimalElt = f64;

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
enum Token {
    Identifier(String),
    Number(Vec<String>),
    Operator(String),
    StringLiteral(String),
}

fn lex(input: &str) -> Result<Vec<Token>> {
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

fn odometer(range: &[usize]) -> Box<dyn Iterator<Item = Vec<usize>>> {
    Box::new(range.iter().map(|&w| 0..w).multi_cartesian_product().fuse())
}

#[cfg(test)]
mod test {
    use crate::odometer;

    #[test]
    fn test_odometer() {
        let mut mp = odometer(&[2, 3, 4]);
        assert_eq!(mp.next(), Some(vec![0, 0, 0]));
        assert_eq!(mp.next(), Some(vec![0, 0, 1]));
        assert_eq!(mp.next(), Some(vec![0, 0, 2]));
        assert_eq!(mp.next(), Some(vec![0, 0, 3]));
        assert_eq!(mp.next(), Some(vec![0, 1, 0]));
        assert_eq!(mp.next(), Some(vec![0, 1, 1]));
        assert_eq!(mp.next(), Some(vec![0, 1, 2]));
        assert_eq!(mp.next(), Some(vec![0, 1, 3]));
        assert_eq!(mp.next(), Some(vec![0, 2, 0]));
        assert_eq!(mp.next(), Some(vec![0, 2, 1]));
        assert_eq!(mp.next(), Some(vec![0, 2, 2]));
        assert_eq!(mp.next(), Some(vec![0, 2, 3]));
        assert_eq!(mp.next(), Some(vec![1, 0, 0]));
        assert_eq!(mp.next(), Some(vec![1, 0, 1]));
        assert_eq!(mp.next(), Some(vec![1, 0, 2]));
        assert_eq!(mp.next(), Some(vec![1, 0, 3]));
        assert_eq!(mp.next(), Some(vec![1, 1, 0]));
        assert_eq!(mp.next(), Some(vec![1, 1, 1]));
        assert_eq!(mp.next(), Some(vec![1, 1, 2]));
        assert_eq!(mp.next(), Some(vec![1, 1, 3]));
        assert_eq!(mp.next(), Some(vec![1, 2, 0]));
        assert_eq!(mp.next(), Some(vec![1, 2, 1]));
        assert_eq!(mp.next(), Some(vec![1, 2, 2]));
        assert_eq!(mp.next(), Some(vec![1, 2, 3]));
        assert_eq!(mp.next(), None);
    }
}

#[derive(Debug, Clone)]
struct GenericArray<T>
where
    T: Copy + Debug,
{
    shape: Vec<usize>,
    data: Vec<T>,
}

impl<T> GenericArray<T>
where
    T: Copy + Debug,
{
    fn get_index(&self, index: &[usize]) -> Option<usize> {
        if index.len() != self.rank() {
            return None;
        }
        Some(zip(index, &self.shape).fold(0, |acc, (i, shape)| acc * shape + i))
    }

    fn get_mut(&mut self, index: &[usize]) -> Option<&mut T> {
        let i = self.get_index(index)?;
        Some(&mut self.data[i])
    }

    fn get(&self, index: &[usize]) -> Option<T> {
        let i = self.get_index(index)?;
        Some(self.data[i])
    }

    pub fn agrees<U>(&self, other: &GenericArray<U>) -> bool
    where
        U: Copy + Debug,
    {
        zip(&self.shape, &other.shape).all(|(a, w)| a == w)
    }

    pub fn rank(&self) -> usize {
        self.shape.len()
    }

    fn generic_map<F, U>(self, f: F) -> GenericArray<U>
    where
        F: Fn(T) -> U,
        U: Copy + Debug,
    {
        GenericArray {
            shape: self.shape,
            data: self.data.into_iter().map(f).collect(),
        }
    }

    fn generic_atom_map_right<F, U, V>(self, w: U, f: F) -> GenericArray<V>
    where
        F: Fn(T, U) -> V,
        U: Copy + Debug,
        V: Copy + Debug,
    {
        GenericArray {
            shape: self.shape,
            data: self.data.into_iter().map(|a| f(a, w)).collect(),
        }
    }

    fn generic_atom_map_left<F, U, V>(self, a: U, f: F) -> GenericArray<V>
    where
        F: Fn(U, T) -> V,
        U: Copy + Debug,
        V: Copy + Debug,
    {
        GenericArray {
            shape: self.shape,
            data: self.data.into_iter().map(|w| f(a, w)).collect(),
        }
    }

    fn generic_agreement_map<F, U, V>(self, other: GenericArray<U>, f: F) -> Option<GenericArray<V>>
    where
        F: Fn(T, U) -> V,
        U: Copy + Debug,
        V: Copy + Debug,
    {
        if !self.agrees(&other) {
            return None;
        }

        let (shape, data) = match self.rank().cmp(&other.rank()) {
            Ordering::Less => todo!(),
            Ordering::Equal => {
                (self.shape.clone(), odometer(&self.shape).map(|index| f(self.get(&index).unwrap(), other.get(&index).unwrap())).collect_vec())
            }
            Ordering::Greater => todo!(),
        };

        Some(GenericArray {
            shape,
            data,
        })

        // let (mut primary, secondary) = if swap { (self, other) } else { (other, self) };
        //
        // if primary.rank() == secondary.rank() {
        //     for full_index in odometer(&primary.shape) {
        //         let a = primary
        //             .get_mut(&full_index)
        //             .expect("Odometer-generated index should be in range");
        //         let w = secondary
        //             .get(&full_index)
        //             .expect("Odometer-generated index should be in range");
        //         if swap {
        //             *a = f(*a, w);
        //         } else {
        //             *a = f(w, *a);
        //         }
        //     }
        // } else {
        //     for leading_index in odometer(&secondary.shape) {
        //         for trailing_index in odometer(&primary.shape[secondary.rank()..]) {
        //             let mut full_index = leading_index.clone();
        //             full_index.extend_from_slice(&trailing_index);
        //             let a = primary
        //                 .get_mut(&full_index)
        //                 .expect("Odometer-generated index should be in range");
        //             let w = secondary
        //                 .get(&leading_index)
        //                 .expect("Odometer-generated index should be in range");
        //             if swap {
        //                 *a = f(*a, w);
        //             } else {
        //                 *a = f(w, *a);
        //             }
        //         }
        //     }
        // }
    }

    pub fn cast<U>(self) -> GenericArray<U>
    where
        U: Copy + Debug + From<T>,
    {
        self.generic_map(Into::into)
    }

    pub fn abs(self) -> Self
    where
        T: Signed,
    {
        self.generic_map(abs)
    }

    pub fn neg(self) -> Self
    where
        T: Signed,
    {
        self.generic_map(|w| w.neg())
    }

    pub fn add_atom<U, V>(self, atom: U) -> GenericArray<V>
    where
        T: Add<U, Output = V>,
        U: Copy,
        V: Copy + Debug,
    {
        self.generic_map(|w| w + atom)
    }

    pub fn add(self, other: GenericArray<T>) -> Option<GenericArray<T>>
    where
        T: Add<Output = T>,
    {
        self.generic_agreement_map(other, |a, w| a + w)
    }

    pub fn sub_atom<U, V>(self, atom: U) -> GenericArray<V>
    where
        T: Sub<U, Output = V>,
        U: Copy,
        V: Copy + Debug,
    {
        self.generic_map(|w| w - atom)
    }

    pub fn sub(self, other: GenericArray<T>) -> Option<GenericArray<T>>
    where
        T: Sub<Output = T>,
    {
        self.generic_agreement_map(other, |a, w| a - w)
    }
}

impl GenericArray<isize> {
    pub fn iota(shape: &[usize]) -> Self {
        GenericArray {
            shape: shape.to_vec(),
            data: (0..shape.iter().product::<usize>())
                .map(|w| w as isize)
                .collect(),
        }
    }
}

impl<T> TryFrom<&[String]> for GenericArray<T>
where
    T: Copy + Debug + FromStr,
    <T as FromStr>::Err: 'static + std::error::Error + Send + Sync,
{
    type Error = anyhow::Error;

    fn try_from(value: &[String]) -> Result<Self> {
        let shape = vec![value.len()];
        let data = value
            .iter()
            .map(|w| {
                w.parse()
                    .with_context(|| anyhow!("Failed to parse {w} as a number."))
            })
            .collect::<Result<_>>()?;
        Ok(Self { shape, data })
    }
}

#[derive(Debug, Clone)]
enum Array {
    Boolean(GenericArray<bool>),
    Integer(GenericArray<IntegerElt>),
    Decimal(GenericArray<DecimalElt>),
}

#[derive(Debug, Copy, Clone)]
enum Atom {
    Boolean(bool),
    Integer(IntegerElt),
    Decimal(DecimalElt),
}

#[derive(Debug, Clone)]
enum Noun {
    Atom(Atom),
    Array(Array),
}

enum GenericMatchingNouns<T>
where
    T: Copy + Debug,
{
    ArrArr(GenericArray<T>, GenericArray<T>),
    ArrAt(GenericArray<T>, T),
    AtArr(T, GenericArray<T>),
    AtAt(T, T),
}

enum ArrayOrAtom<T>
where
    T: Copy + Debug,
{
    Array(GenericArray<T>),
    Atom(T),
}

impl<T> GenericMatchingNouns<T>
where
    T: Copy + Debug,
{
    fn dyad<F, U>(self, f: F) -> Option<ArrayOrAtom<U>>
    where
        F: Fn(T, T) -> U,
        U: Copy + Debug,
    {
        use GenericMatchingNouns::*;
        Some(match self {
            ArrArr(a, w) => ArrayOrAtom::Array(a.generic_agreement_map(w, f)?),
            ArrAt(a, w) => ArrayOrAtom::Array(a.generic_atom_map_right(w, f)),
            AtArr(a, w) => ArrayOrAtom::Array(w.generic_atom_map_left(a, f)),
            AtAt(a, w) => ArrayOrAtom::Atom(f(a, w)),
        })
    }
}

enum MatchingNouns {
    Boolean(GenericMatchingNouns<bool>),
    Integer(GenericMatchingNouns<IntegerElt>),
    Decimal(GenericMatchingNouns<DecimalElt>),
}

impl Noun {
    fn try_promote_pair(a: Noun, w: Noun) -> Option<MatchingNouns> {
        use Array as Arr;
        use Atom as At;
        use GenericMatchingNouns::*;
        use MatchingNouns as MN;
        use Noun as N;
        match (a, w) {
            // Array - Array
            // Identical cases
            (N::Array(Arr::Boolean(a)), N::Array(Arr::Boolean(w))) => {
                Some(MN::Boolean(ArrArr(a, w)))
            }
            (N::Array(Arr::Integer(a)), N::Array(Arr::Integer(w))) => {
                Some(MN::Integer(ArrArr(a, w)))
            }
            (N::Array(Arr::Decimal(a)), N::Array(Arr::Decimal(w))) => {
                Some(MN::Decimal(ArrArr(a, w)))
            }
            // Boolean left
            (N::Array(Arr::Boolean(a)), N::Array(Arr::Integer(w))) => {
                Some(MN::Integer(ArrArr(a.cast(), w)))
            }
            (N::Array(Arr::Boolean(a)), N::Array(Arr::Decimal(w))) => {
                Some(MN::Decimal(ArrArr(a.cast::<IntegerElt>().cast(), w)))
            }
            // Integer left
            (N::Array(Arr::Integer(a)), N::Array(Arr::Boolean(w))) => {
                Some(MN::Integer(ArrArr(a, w.cast())))
            }
            (N::Array(Arr::Integer(a)), N::Array(Arr::Decimal(w))) => {
                Some(MN::Decimal(ArrArr(a.cast(), w)))
            }
            // Decimal left
            (N::Array(Arr::Decimal(a)), N::Array(Arr::Boolean(w))) => {
                Some(MN::Decimal(ArrArr(a, w.cast::<IntegerElt>().cast())))
            }
            (N::Array(Arr::Decimal(a)), N::Array(Arr::Integer(w))) => {
                Some(MN::Decimal(ArrArr(a, w.cast())))
            }

            // Array - Atom
            // Identical cases
            (N::Array(Arr::Boolean(a)), N::Atom(At::Boolean(w))) => Some(MN::Boolean(ArrAt(a, w))),
            (N::Array(Arr::Integer(a)), N::Atom(At::Integer(w))) => Some(MN::Integer(ArrAt(a, w))),
            (N::Array(Arr::Decimal(a)), N::Atom(At::Decimal(w))) => Some(MN::Decimal(ArrAt(a, w))),
            // Boolean left
            (N::Array(Arr::Boolean(a)), N::Atom(At::Integer(w))) => {
                Some(MN::Integer(ArrAt(a.cast(), w)))
            }
            (N::Array(Arr::Boolean(a)), N::Atom(At::Decimal(w))) => {
                Some(MN::Decimal(ArrAt(a.cast::<IntegerElt>().cast(), w)))
            }
            // Integer left
            (N::Array(Arr::Integer(a)), N::Atom(At::Boolean(w))) => {
                Some(MN::Integer(ArrAt(a, w as IntegerElt)))
            }
            (N::Array(Arr::Integer(a)), N::Atom(At::Decimal(w))) => {
                Some(MN::Decimal(ArrAt(a.cast(), w)))
            }
            // Decimal left
            (N::Array(Arr::Decimal(a)), N::Atom(At::Boolean(w))) => {
                Some(MN::Decimal(ArrAt(a, w as IntegerElt as DecimalElt)))
            }
            (N::Array(Arr::Decimal(a)), N::Atom(At::Integer(w))) => {
                Some(MN::Decimal(ArrAt(a, w as DecimalElt)))
            }

            // Atom - Array
            // Identical cases
            (N::Atom(At::Boolean(a)), N::Array(Arr::Boolean(w))) => Some(MN::Boolean(AtArr(a, w))),
            (N::Atom(At::Integer(a)), N::Array(Arr::Integer(w))) => Some(MN::Integer(AtArr(a, w))),
            (N::Atom(At::Decimal(a)), N::Array(Arr::Decimal(w))) => Some(MN::Decimal(AtArr(a, w))),
            // Boolean left
            (N::Atom(At::Boolean(a)), N::Array(Arr::Integer(w))) => {
                Some(MN::Integer(AtArr(a as IntegerElt, w)))
            }
            (N::Atom(At::Boolean(a)), N::Array(Arr::Decimal(w))) => {
                Some(MN::Decimal(AtArr(a as IntegerElt as DecimalElt, w)))
            }
            // Integer left
            (N::Atom(At::Integer(a)), N::Array(Arr::Boolean(w))) => {
                Some(MN::Integer(AtArr(a, w.cast())))
            }
            (N::Atom(At::Integer(a)), N::Array(Arr::Decimal(w))) => {
                Some(MN::Decimal(AtArr(a as DecimalElt, w)))
            }
            // Decimal left
            (N::Atom(At::Decimal(a)), N::Array(Arr::Boolean(w))) => {
                Some(MN::Decimal(AtArr(a, w.cast::<IntegerElt>().cast())))
            }
            (N::Atom(At::Decimal(a)), N::Array(Arr::Integer(w))) => {
                Some(MN::Decimal(AtArr(a, w.cast())))
            }

            // Atom - Atom
            // Identical cases
            (N::Atom(At::Boolean(a)), N::Atom(At::Boolean(w))) => Some(MN::Boolean(AtAt(a, w))),
            (N::Atom(At::Integer(a)), N::Atom(At::Integer(w))) => Some(MN::Integer(AtAt(a, w))),
            (N::Atom(At::Decimal(a)), N::Atom(At::Decimal(w))) => Some(MN::Decimal(AtAt(a, w))),
            // Boolean left
            (N::Atom(At::Boolean(a)), N::Atom(At::Integer(w))) => {
                Some(MN::Integer(AtAt(a as IntegerElt, w)))
            }
            (N::Atom(At::Boolean(a)), N::Atom(At::Decimal(w))) => {
                Some(MN::Decimal(AtAt(a as IntegerElt as DecimalElt, w)))
            }
            // Integer left
            (N::Atom(At::Integer(a)), N::Atom(At::Boolean(w))) => {
                Some(MN::Integer(AtAt(a, w as IntegerElt)))
            }
            (N::Atom(At::Integer(a)), N::Atom(At::Decimal(w))) => {
                Some(MN::Decimal(AtAt(a as DecimalElt, w)))
            }
            // Decimal left
            (N::Atom(At::Decimal(a)), N::Atom(At::Boolean(w))) => {
                Some(MN::Decimal(AtAt(a, w as IntegerElt as DecimalElt)))
            }
            (N::Atom(At::Decimal(a)), N::Atom(At::Integer(w))) => {
                Some(MN::Decimal(AtAt(a, w as DecimalElt)))
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Variable {
    Noun(Noun),
    Verb(String),
}

mod monads {
    use crate::Noun;
    use anyhow::Result;

    pub(super) fn same(w: Noun) -> Result<Noun> {
        Ok(w)
    }
}

type MonadFn = fn(Noun) -> Result<Noun>;
static MONADS: phf::Map<&'static str, MonadFn> = phf_map! {
    "]" => monads::same,
    "[" => monads::same,
};

mod dyads {
    use crate::{Array, ArrayOrAtom, Atom, IntegerElt, MatchingNouns, Noun};
    use anyhow::{Context, Result};

    pub(super) fn same_w(_: Noun, w: Noun) -> Result<Noun> {
        Ok(w)
    }

    pub(super) fn same_a(a: Noun, _: Noun) -> Result<Noun> {
        Ok(a)
    }

    pub(super) fn add(a: Noun, w: Noun) -> Result<Noun> {
        use Array as Arr;
        use ArrayOrAtom as AoA;
        use Atom as At;
        use MatchingNouns as MN;
        use Noun as N;
        Ok(
            match N::try_promote_pair(a, w).context("failed to promote")? {
                MN::Boolean(nouns) => match nouns
                    .dyad(|a, w| a as IntegerElt + w as IntegerElt)
                    .context("dyad failure")?
                {
                    AoA::Array(arr) => N::Array(Arr::Integer(arr)),
                    AoA::Atom(at) => N::Atom(At::Integer(at)),
                },
                MN::Integer(nouns) => match nouns.dyad(|a, w| a + w).context("dyad failure")? {
                    AoA::Array(arr) => N::Array(Arr::Integer(arr)),
                    AoA::Atom(at) => N::Atom(At::Integer(at)),
                },
                MN::Decimal(nouns) => match nouns.dyad(|a, w| a + w).context("dyad failure")? {
                    AoA::Array(arr) => N::Array(Arr::Decimal(arr)),
                    AoA::Atom(at) => N::Atom(At::Decimal(at)),
                },
            },
        )
    }
}

type DyadFn = fn(Noun, Noun) -> Result<Noun>;
static DYADS: phf::Map<&'static str, DyadFn> = phf_map! {
    "]" => dyads::same_w,
    "[" => dyads::same_a,
    "+" => dyads::add,
};

fn interpret(mut csl: Vec<Token>, env: &mut HashMap<String, Variable>) -> Result<Option<Noun>> {
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

fn main() -> Result<()> {
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
