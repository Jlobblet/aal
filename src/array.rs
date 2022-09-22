use anyhow::Result;
use anyhow::{anyhow, Context};
use itertools::Itertools;
use num_traits::{abs, Signed};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::iter::zip;
use std::ops::{Add, Neg, Sub};
use std::str::FromStr;

pub type IntegerElt = i32;
pub type DecimalElt = f64;

fn odometer(range: &[usize]) -> Box<dyn Iterator<Item = Vec<usize>>> {
    Box::new(range.iter().map(|&w| 0..w).multi_cartesian_product().fuse())
}

#[cfg(test)]
mod test {
    use crate::array::odometer;

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
pub struct GenericArray<T>
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

    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    pub fn raw_data(&self) -> &[T] {
        &self.data
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
            Ordering::Equal => (
                self.shape.clone(),
                odometer(&self.shape)
                    .map(|index| f(self.get(&index).unwrap(), other.get(&index).unwrap()))
                    .collect_vec(),
            ),
            Ordering::Greater => todo!(),
        };

        Some(GenericArray { shape, data })

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

impl GenericArray<IntegerElt> {
    pub fn iota(shape: &[IntegerElt]) -> Self {
        GenericArray {
            shape: shape.iter().map(|&w| w as usize).collect(),
            data: (0..shape.iter().product::<IntegerElt>())
                .map(|w| w as IntegerElt)
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

    fn try_from(value: &[String]) -> anyhow::Result<Self> {
        let shape = vec![value.len()];
        let data = value
            .iter()
            .map(|w| {
                w.parse()
                    .with_context(|| anyhow!("Failed to parse {w} as a number."))
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(Self { shape, data })
    }
}

#[derive(Debug, Clone)]
pub enum Array {
    Boolean(GenericArray<bool>),
    Integer(GenericArray<IntegerElt>),
    Decimal(GenericArray<DecimalElt>),
}

#[derive(Debug, Copy, Clone)]
pub enum Atom {
    Boolean(bool),
    Integer(IntegerElt),
    Decimal(DecimalElt),
}

#[derive(Debug, Clone)]
pub enum Noun {
    Array(Array),
    Atom(Atom),
}

impl Noun {
    pub fn map<FB, FI, FD, V>(self, b: FB, i: FI, d: FD) -> ArrayOrAtom<V>
    where
        FB: Fn(bool) -> V,
        FI: Fn(IntegerElt) -> V,
        FD: Fn(DecimalElt) -> V,
        V: Copy + Debug,
    {
        use Array as Arr;
        use ArrayOrAtom as AoA;
        use Atom as At;
        use Noun as N;
        match self {
            N::Array(w) => AoA::Array(match w {
                Arr::Boolean(w) => w.generic_map(b),
                Arr::Integer(w) => w.generic_map(i),
                Arr::Decimal(w) => w.generic_map(d),
            }),
            N::Atom(w) => AoA::Atom(match w {
                At::Boolean(w) => b(w),
                At::Integer(w) => i(w),
                At::Decimal(w) => d(w),
            }),
        }
    }

    pub fn to_boolean(self) -> ArrayOrAtom<bool> {
        self.map(|w| w, |w| w != 0, |w| w != 0.0)
    }
}

impl From<ArrayOrAtom<bool>> for Noun {
    fn from(w: ArrayOrAtom<bool>) -> Self {
        match w {
            ArrayOrAtom::Array(w) => Noun::Array(Array::Boolean(w)),
            ArrayOrAtom::Atom(w) => Noun::Atom(Atom::Boolean(w)),
        }
    }
}

impl From<ArrayOrAtom<IntegerElt>> for Noun {
    fn from(w: ArrayOrAtom<IntegerElt>) -> Self {
        match w {
            ArrayOrAtom::Array(w) => Noun::Array(Array::Integer(w)),
            ArrayOrAtom::Atom(w) => Noun::Atom(Atom::Integer(w)),
        }
    }
}

impl From<ArrayOrAtom<DecimalElt>> for Noun {
    fn from(w: ArrayOrAtom<DecimalElt>) -> Self {
        match w {
            ArrayOrAtom::Array(w) => Noun::Array(Array::Decimal(w)),
            ArrayOrAtom::Atom(w) => Noun::Atom(Atom::Decimal(w)),
        }
    }
}

pub enum GenericMatchingNouns<T>
where
    T: Copy + Debug,
{
    ArrArr(GenericArray<T>, GenericArray<T>),
    ArrAt(GenericArray<T>, T),
    AtArr(T, GenericArray<T>),
    AtAt(T, T),
}

impl<T> From<(ArrayOrAtom<T>, ArrayOrAtom<T>)> for GenericMatchingNouns<T>
where
    T: Copy + Debug,
{
    fn from(aw: (ArrayOrAtom<T>, ArrayOrAtom<T>)) -> Self {
        use ArrayOrAtom as AoA;
        use GenericMatchingNouns::*;
        match aw {
            (AoA::Array(a), AoA::Array(w)) => ArrArr(a, w),
            (AoA::Array(a), AoA::Atom(w)) => ArrAt(a, w),
            (AoA::Atom(a), AoA::Array(w)) => AtArr(a, w),
            (AoA::Atom(a), AoA::Atom(w)) => AtAt(a, w),
        }
    }
}

pub enum ArrayOrAtom<T>
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
    pub fn dyad<F, U>(self, f: F) -> Option<ArrayOrAtom<U>>
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

pub enum MatchingNouns {
    Boolean(GenericMatchingNouns<bool>),
    Integer(GenericMatchingNouns<IntegerElt>),
    Decimal(GenericMatchingNouns<DecimalElt>),
}

impl MatchingNouns {
    pub fn dyad<FB, OB, FI, OI, FD, OD>(self, b: FB, i: FI, d: FD) -> Result<Noun>
    where
        FB: Fn(bool, bool) -> OB,
        FI: Fn(IntegerElt, IntegerElt) -> OI,
        FD: Fn(DecimalElt, DecimalElt) -> OD,
        OB: Copy + Debug,
        OI: Copy + Debug,
        OD: Copy + Debug,
        Noun: From<ArrayOrAtom<OB>> + From<ArrayOrAtom<OD>> + From<ArrayOrAtom<OI>>,
    {
        use crate::array::Array as Arr;
        use crate::array::ArrayOrAtom as AoA;
        use crate::array::Atom as At;
        use crate::array::MatchingNouns as MN;
        use crate::array::Noun as N;
        Ok(match self {
            MN::Boolean(nouns) => nouns.dyad(b).context("dyad failure")?.into(),
            MN::Integer(nouns) => nouns.dyad(i).context("dyad failure")?.into(),
            MN::Decimal(nouns) => nouns.dyad(d).context("dyad failure")?.into(),
        })
    }
}

impl Noun {
    pub fn try_promote_pair(a: Noun, w: Noun) -> Option<MatchingNouns> {
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
