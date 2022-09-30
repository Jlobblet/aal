use crate::arrays;
use crate::arrays::IntegerElt;
use anyhow::{anyhow, Context};
use itertools::Itertools;
use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::iter::zip;
use std::str::FromStr;

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

    #[allow(unused)]
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

    #[inline]
    pub fn rank(&self) -> usize {
        self.shape.len()
    }

    #[allow(unused)]
    #[inline]
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    #[inline]
    pub fn raw_data(&self) -> &[T] {
        &self.data
    }

    pub fn map<F, U>(self, f: F) -> GenericArray<U>
    where
        F: Fn(T) -> U,
        U: Copy + Debug,
    {
        GenericArray {
            shape: self.shape,
            data: self.data.into_iter().map(f).collect(),
        }
    }

    pub(crate) fn atom_map_right<F, U, V>(self, w: U, f: F) -> GenericArray<V>
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

    pub(crate) fn atom_map_left<F, U, V>(self, a: U, f: F) -> GenericArray<V>
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

    pub(crate) fn agreement_map<F, U, V>(
        self,
        other: GenericArray<U>,
        f: F,
    ) -> anyhow::Result<GenericArray<V>>
    where
        F: Fn(T, U) -> V,
        U: Copy + Debug,
        V: Copy + Debug,
    {
        if !self.agrees(&other) {
            return Err(anyhow!(
                "Incompatible arrays shapes for dyadic operation: {} and {}",
                self.shape.iter().map(|w| w.to_string()).join(" "),
                other.shape.iter().map(|w| w.to_string()).join(" ")
            ));
        }

        let (shape, data) = match self.rank().cmp(&other.rank()) {
            Ordering::Less => {
                let d = other.data.len() / self.data.len();
                let data = other
                    .data
                    .into_iter()
                    .enumerate()
                    .map(|(i, w)| f(self.data[i / d], w))
                    .collect();
                (other.shape, data)
            }
            Ordering::Equal => (
                self.shape.clone(),
                self.data
                    .into_iter()
                    .zip(other.data.into_iter())
                    .map(|(a, w)| f(a, w))
                    .collect(),
            ),
            Ordering::Greater => {
                let d = self.data.len() / other.data.len();
                let data = self
                    .data
                    .into_iter()
                    .enumerate()
                    .map(|(i, a)| f(a, other.data[i / d]))
                    .collect();
                (self.shape, data)
            }
        };

        Ok(GenericArray { shape, data })
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

impl<T, S> TryFrom<&[S]> for GenericArray<T>
where
    T: Copy + Debug + FromStr,
    <T as FromStr>::Err: 'static + std::error::Error + Send + Sync,
    S: AsRef<str> + Display,
{
    type Error = anyhow::Error;

    fn try_from(value: &[S]) -> anyhow::Result<Self> {
        let shape = vec![value.len()];
        let data = value
            .iter()
            .map(|w| {
                w.as_ref()
                    .parse()
                    .with_context(|| anyhow!("Failed to parse {w} as a number."))
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(Self { shape, data })
    }
}
