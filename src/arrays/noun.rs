use crate::arrays::array::Array;
use crate::arrays::array_or_atom::ArrayOrAtom;
use crate::arrays::atom::Atom;
use crate::arrays::generic_array::GenericArray;
use crate::arrays::matching_nouns::MatchingNouns;
use crate::arrays::{DecimalElt, IntegerElt};
use anyhow::anyhow;
use std::fmt::Debug;

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
        use crate::arrays::array::Array as Arr;
        use crate::arrays::array_or_atom::ArrayOrAtom as AoA;
        use crate::arrays::atom::Atom as At;
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

    pub fn into_boolean(self) -> ArrayOrAtom<bool> {
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
            ArrayOrAtom::Array(w) => Noun::Array(w.into()),
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

impl From<GenericArray<bool>> for Noun {
    fn from(w: GenericArray<bool>) -> Self {
        Noun::Array(Array::Boolean(w))
    }
}

impl From<GenericArray<IntegerElt>> for Noun {
    fn from(w: GenericArray<IntegerElt>) -> Self {
        Noun::Array(Array::Integer(w))
    }
}

impl From<GenericArray<DecimalElt>> for Noun {
    fn from(w: GenericArray<DecimalElt>) -> Self {
        Noun::Array(Array::Decimal(w))
    }
}

impl From<bool> for Noun {
    fn from(w: bool) -> Self {
        Noun::Atom(Atom::Boolean(w))
    }
}

impl From<IntegerElt> for Noun {
    fn from(w: IntegerElt) -> Self {
        Noun::Atom(Atom::Integer(w))
    }
}

impl From<DecimalElt> for Noun {
    fn from(w: DecimalElt) -> Self {
        Noun::Atom(Atom::Decimal(w))
    }
}

impl Noun {
    pub fn try_promote_pair(a: Noun, w: Noun) -> anyhow::Result<MatchingNouns> {
        use crate::arrays::array::Array as Arr;
        use crate::arrays::atom::Atom as At;
        use crate::arrays::generic_matching_nouns::GenericMatchingNouns::*;
        use crate::arrays::matching_nouns::MatchingNouns as MN;
        use Noun as N;
        match (a, w) {
            // Array - Array
            // Identical cases
            (N::Array(Arr::Boolean(a)), N::Array(Arr::Boolean(w))) => Ok(MN::Boolean(ArrArr(a, w))),
            (N::Array(Arr::Integer(a)), N::Array(Arr::Integer(w))) => Ok(MN::Integer(ArrArr(a, w))),
            (N::Array(Arr::Decimal(a)), N::Array(Arr::Decimal(w))) => Ok(MN::Decimal(ArrArr(a, w))),
            // Boolean left
            (N::Array(Arr::Boolean(a)), N::Array(Arr::Integer(w))) => {
                Ok(MN::Integer(ArrArr(a.cast(), w)))
            }
            (N::Array(Arr::Boolean(a)), N::Array(Arr::Decimal(w))) => {
                Ok(MN::Decimal(ArrArr(a.cast::<IntegerElt>().cast(), w)))
            }
            // Integer left
            (N::Array(Arr::Integer(a)), N::Array(Arr::Boolean(w))) => {
                Ok(MN::Integer(ArrArr(a, w.cast())))
            }
            (N::Array(Arr::Integer(a)), N::Array(Arr::Decimal(w))) => {
                Ok(MN::Decimal(ArrArr(a.cast(), w)))
            }
            // Decimal left
            (N::Array(Arr::Decimal(a)), N::Array(Arr::Boolean(w))) => {
                Ok(MN::Decimal(ArrArr(a, w.cast::<IntegerElt>().cast())))
            }
            (N::Array(Arr::Decimal(a)), N::Array(Arr::Integer(w))) => {
                Ok(MN::Decimal(ArrArr(a, w.cast())))
            }

            // Array - Atom
            // Identical cases
            (N::Array(Arr::Boolean(a)), N::Atom(At::Boolean(w))) => Ok(MN::Boolean(ArrAt(a, w))),
            (N::Array(Arr::Integer(a)), N::Atom(At::Integer(w))) => Ok(MN::Integer(ArrAt(a, w))),
            (N::Array(Arr::Decimal(a)), N::Atom(At::Decimal(w))) => Ok(MN::Decimal(ArrAt(a, w))),
            // Boolean left
            (N::Array(Arr::Boolean(a)), N::Atom(At::Integer(w))) => {
                Ok(MN::Integer(ArrAt(a.cast(), w)))
            }
            (N::Array(Arr::Boolean(a)), N::Atom(At::Decimal(w))) => {
                Ok(MN::Decimal(ArrAt(a.cast::<IntegerElt>().cast(), w)))
            }
            // Integer left
            (N::Array(Arr::Integer(a)), N::Atom(At::Boolean(w))) => {
                Ok(MN::Integer(ArrAt(a, w as IntegerElt)))
            }
            (N::Array(Arr::Integer(a)), N::Atom(At::Decimal(w))) => {
                Ok(MN::Decimal(ArrAt(a.cast(), w)))
            }
            // Decimal left
            (N::Array(Arr::Decimal(a)), N::Atom(At::Boolean(w))) => {
                Ok(MN::Decimal(ArrAt(a, w as IntegerElt as DecimalElt)))
            }
            (N::Array(Arr::Decimal(a)), N::Atom(At::Integer(w))) => {
                Ok(MN::Decimal(ArrAt(a, w as DecimalElt)))
            }

            // Atom - Array
            // Identical cases
            (N::Atom(At::Boolean(a)), N::Array(Arr::Boolean(w))) => Ok(MN::Boolean(AtArr(a, w))),
            (N::Atom(At::Integer(a)), N::Array(Arr::Integer(w))) => Ok(MN::Integer(AtArr(a, w))),
            (N::Atom(At::Decimal(a)), N::Array(Arr::Decimal(w))) => Ok(MN::Decimal(AtArr(a, w))),
            // Boolean left
            (N::Atom(At::Boolean(a)), N::Array(Arr::Integer(w))) => {
                Ok(MN::Integer(AtArr(a as IntegerElt, w)))
            }
            (N::Atom(At::Boolean(a)), N::Array(Arr::Decimal(w))) => {
                Ok(MN::Decimal(AtArr(a as IntegerElt as DecimalElt, w)))
            }
            // Integer left
            (N::Atom(At::Integer(a)), N::Array(Arr::Boolean(w))) => {
                Ok(MN::Integer(AtArr(a, w.cast())))
            }
            (N::Atom(At::Integer(a)), N::Array(Arr::Decimal(w))) => {
                Ok(MN::Decimal(AtArr(a as DecimalElt, w)))
            }
            // Decimal left
            (N::Atom(At::Decimal(a)), N::Array(Arr::Boolean(w))) => {
                Ok(MN::Decimal(AtArr(a, w.cast::<IntegerElt>().cast())))
            }
            (N::Atom(At::Decimal(a)), N::Array(Arr::Integer(w))) => {
                Ok(MN::Decimal(AtArr(a, w.cast())))
            }

            // Atom - Atom
            // Identical cases
            (N::Atom(At::Boolean(a)), N::Atom(At::Boolean(w))) => Ok(MN::Boolean(AtAt(a, w))),
            (N::Atom(At::Integer(a)), N::Atom(At::Integer(w))) => Ok(MN::Integer(AtAt(a, w))),
            (N::Atom(At::Decimal(a)), N::Atom(At::Decimal(w))) => Ok(MN::Decimal(AtAt(a, w))),
            // Boolean left
            (N::Atom(At::Boolean(a)), N::Atom(At::Integer(w))) => {
                Ok(MN::Integer(AtAt(a as IntegerElt, w)))
            }
            (N::Atom(At::Boolean(a)), N::Atom(At::Decimal(w))) => {
                Ok(MN::Decimal(AtAt(a as IntegerElt as DecimalElt, w)))
            }
            // Integer left
            (N::Atom(At::Integer(a)), N::Atom(At::Boolean(w))) => {
                Ok(MN::Integer(AtAt(a, w as IntegerElt)))
            }
            (N::Atom(At::Integer(a)), N::Atom(At::Decimal(w))) => {
                Ok(MN::Decimal(AtAt(a as DecimalElt, w)))
            }
            // Decimal left
            (N::Atom(At::Decimal(a)), N::Atom(At::Boolean(w))) => {
                Ok(MN::Decimal(AtAt(a, w as IntegerElt as DecimalElt)))
            }
            (N::Atom(At::Decimal(a)), N::Atom(At::Integer(w))) => {
                Ok(MN::Decimal(AtAt(a, w as DecimalElt)))
            }
            #[allow(unreachable_patterns)]
            (a, w) => Err(anyhow!(
                "Incompatible types for promotion: {:?} and {:?}",
                a,
                w
            )),
        }
    }
}
