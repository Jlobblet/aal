use crate::arrays::array_or_atom::ArrayOrAtom;
use crate::arrays::generic_matching_nouns::GenericMatchingNouns;
use crate::arrays::noun::Noun;
use crate::arrays::{DecimalElt, IntegerElt};
use std::fmt::Debug;

pub enum MatchingNouns {
    Boolean(GenericMatchingNouns<bool>),
    Integer(GenericMatchingNouns<IntegerElt>),
    Decimal(GenericMatchingNouns<DecimalElt>),
}

impl From<GenericMatchingNouns<bool>> for MatchingNouns {
    fn from(w: GenericMatchingNouns<bool>) -> Self {
        Self::Boolean(w)
    }
}

impl From<GenericMatchingNouns<IntegerElt>> for MatchingNouns {
    fn from(w: GenericMatchingNouns<IntegerElt>) -> Self {
        Self::Integer(w)
    }
}

impl From<GenericMatchingNouns<DecimalElt>> for MatchingNouns {
    fn from(w: GenericMatchingNouns<DecimalElt>) -> Self {
        Self::Decimal(w)
    }
}

impl MatchingNouns {
    pub fn dyad<FB, OB, FI, OI, FD, OD>(self, b: FB, i: FI, d: FD) -> anyhow::Result<Noun>
    where
        FB: Fn(bool, bool) -> OB,
        FI: Fn(IntegerElt, IntegerElt) -> OI,
        FD: Fn(DecimalElt, DecimalElt) -> OD,
        OB: Copy + Debug,
        OI: Copy + Debug,
        OD: Copy + Debug,
        Noun: From<ArrayOrAtom<OB>> + From<ArrayOrAtom<OD>> + From<ArrayOrAtom<OI>>,
    {
        use crate::arrays::matching_nouns::MatchingNouns as MN;
        use anyhow::Context;
        Ok(match self {
            MN::Boolean(nouns) => nouns.dyad(b).context("dyad failure")?.into(),
            MN::Integer(nouns) => nouns.dyad(i).context("dyad failure")?.into(),
            MN::Decimal(nouns) => nouns.dyad(d).context("dyad failure")?.into(),
        })
    }
}
