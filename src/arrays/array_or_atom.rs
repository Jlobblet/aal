use crate::arrays::generic_array::GenericArray;
use std::fmt::Debug;

pub enum ArrayOrAtom<T>
where
    T: Copy + Debug,
{
    Array(GenericArray<T>),
    Atom(T),
}

impl<T> From<T> for ArrayOrAtom<T>
where
    T: Copy + Debug,
{
    fn from(w: T) -> Self {
        ArrayOrAtom::Atom(w)
    }
}

impl<T> From<GenericArray<T>> for ArrayOrAtom<T>
where
    T: Copy + Debug,
{
    fn from(w: GenericArray<T>) -> Self {
        ArrayOrAtom::Array(w)
    }
}
