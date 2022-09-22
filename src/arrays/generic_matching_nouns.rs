use crate::arrays::array_or_atom::ArrayOrAtom;
use crate::arrays::generic_array::GenericArray;
use anyhow::Result;
use std::fmt::Debug;

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
        use crate::arrays::array_or_atom::ArrayOrAtom as AoA;
        use GenericMatchingNouns::*;
        match aw {
            (AoA::Array(a), AoA::Array(w)) => ArrArr(a, w),
            (AoA::Array(a), AoA::Atom(w)) => ArrAt(a, w),
            (AoA::Atom(a), AoA::Array(w)) => AtArr(a, w),
            (AoA::Atom(a), AoA::Atom(w)) => AtAt(a, w),
        }
    }
}

impl<T> GenericMatchingNouns<T>
where
    T: Copy + Debug,
{
    pub fn dyad<F, U>(self, f: F) -> Result<ArrayOrAtom<U>>
    where
        F: Fn(T, T) -> U,
        U: Copy + Debug,
    {
        use GenericMatchingNouns::*;
        Ok(match self {
            ArrArr(a, w) => ArrayOrAtom::Array(a.agreement_map(w, f)?),
            ArrAt(a, w) => ArrayOrAtom::Array(a.atom_map_right(w, f)),
            AtArr(a, w) => ArrayOrAtom::Array(w.atom_map_left(a, f)),
            AtAt(a, w) => ArrayOrAtom::Atom(f(a, w)),
        })
    }
}
