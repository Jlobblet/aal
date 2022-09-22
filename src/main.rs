use std::fmt::Debug;
use anyhow::Result;
use itertools::Itertools;
use num_traits::{abs, Signed};
use std::io::{stdin, stdout, Write};
use std::iter::zip;
use std::ops::{Add, Sub};

trait Boxable {
    fn boxed(self) -> Box<Self>;
}

impl<T> Boxable for T {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

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

fn lex(input: &str) -> Vec<Token> {
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
                let word_index = word_index.expect("should have a word_index when emitting");
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
                            _ => panic!("oh no, what have we done {:?}", current_state),
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
    output
}

fn odometer(range: &[usize]) -> Box<dyn Iterator<Item = Vec<usize>>> {
    range
        .iter()
        .map(|&w| 0..w)
        .multi_cartesian_product()
        .fuse()
        .boxed()
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

// enum Array {
//     Integral(GenericArray<isize>),
//     Decimal(GenericArray<f64>),
//     Boolean(GenericArray<bool>),
// }

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

    fn generic_agreement_map<F>(self, other: GenericArray<T>, f: F) -> Option<Self>
    where
        F: Fn(T, T) -> T,
    {
        if !self.agrees(&other) {
            return None;
        }

        let swap = self.rank() >= other.rank();

        let (mut primary, secondary) = if swap {
            (self, other)
        } else {
            (other, self)
        };

        if primary.rank() == secondary.rank() {
            for full_index in odometer(&primary.shape) {
                let a = primary
                    .get_mut(&full_index)
                    .expect("Odometer-generated index should be in range");
                let w = secondary
                    .get(&full_index)
                    .expect("Odometer-generated index should be in range");
                if swap {
                    *a = f(*a, w);
                } else {
                    *a = f(w, *a);
                }
            }
        } else {
            for leading_index in odometer(&secondary.shape) {
                for trailing_index in odometer(&primary.shape[secondary.rank()..]) {
                    let mut full_index = leading_index.clone();
                    full_index.extend_from_slice(&trailing_index);
                    let a = primary
                        .get_mut(&full_index)
                        .expect("Odometer-generated index should be in range");
                    let w = secondary
                        .get(&leading_index)
                        .expect("Odometer-generated index should be in range");
                    if swap {
                        *a = f(*a, w);
                    } else {
                        *a = f(w, *a);
                    }
                }
            }
        }

        Some(primary)
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

    pub fn sub(self, other: GenericArray<T>) -> Option<GenericArray<T>>
    where
        T: Sub<Output = T>,
    {
        self.generic_agreement_map(other, |a, w| a - w)
    }

    pub fn sub_atom<U, V>(self, atom: U) -> GenericArray<V>
    where
        T: Sub<U, Output = V>,
        U: Copy,
        V: Copy + Debug,
    {
        self.generic_map(|w| w - atom)
    }
}

impl GenericArray<isize> {
    pub fn iota(shape: &[usize]) -> Self
    {
        GenericArray {
            shape: shape.to_vec(),
            data: (0..shape.iter().product::<usize>())
                .map(|w| w as isize)
                .collect(),
        }
    }
}

fn main() {
    let arr1: GenericArray<isize> = GenericArray::iota(&[2, 3, 4]);
    dbg!(&arr1);

    let arr2: GenericArray<isize> = GenericArray {
        shape: vec![2],
        data: vec![100, 200],
    };

    let arr3 = arr2.sub(arr1).unwrap();
    dbg!(arr3);
}

// fn main() -> Result<()> {
//     let mut buffer = String::new();
//     loop {
//         buffer.clear();
//         print!("    ");
//         stdout().flush()?;
//         stdin().read_line(&mut buffer)?;
//         println!("{:?}", lex(&buffer));
//     }
// }
