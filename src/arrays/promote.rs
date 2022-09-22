use crate::arrays::array_or_atom::ArrayOrAtom;
use crate::arrays::generic_array::GenericArray;
use crate::arrays::generic_matching_nouns::GenericMatchingNouns;
use crate::arrays::{DecimalElt, IntegerElt};
use std::fmt::Debug;

pub trait Promote<T> {
    fn promote(self) -> T;
}

impl Promote<IntegerElt> for bool {
    #[inline]
    fn promote(self) -> IntegerElt {
        self as IntegerElt
    }
}

impl Promote<DecimalElt> for bool {
    #[inline]
    fn promote(self) -> DecimalElt {
        self as IntegerElt as DecimalElt
    }
}

impl Promote<DecimalElt> for IntegerElt {
    #[inline]
    fn promote(self) -> DecimalElt {
        self as DecimalElt
    }
}

impl<T, U> Promote<GenericArray<U>> for GenericArray<T>
where
    T: Copy + Debug + Promote<U>,
    U: Copy + Debug,
{
    #[inline]
    fn promote(self) -> GenericArray<U> {
        self.map(Promote::promote)
    }
}

impl<T, U> Promote<ArrayOrAtom<U>> for ArrayOrAtom<T>
where
    T: Copy + Debug + Promote<U>,
    U: Copy + Debug,
{
    #[inline]
    fn promote(self) -> ArrayOrAtom<U> {
        use ArrayOrAtom::*;
        match self {
            Atom(w) => Atom(w.promote()),
            Array(w) => Array(w.promote()),
        }
    }
}

impl<T, U> Promote<GenericMatchingNouns<U>> for GenericMatchingNouns<T>
where
    T: Copy + Debug + Promote<U>,
    U: Copy + Debug,
{
    #[inline]
    fn promote(self) -> GenericMatchingNouns<U> {
        use GenericMatchingNouns::*;
        match self {
            ArrArr(a, w) => ArrArr(a.promote(), w.promote()),
            ArrAt(a, w) => ArrAt(a.promote(), w.promote()),
            AtArr(a, w) => AtArr(a.promote(), w.promote()),
            AtAt(a, w) => AtAt(a.promote(), w.promote()),
        }
    }
}
