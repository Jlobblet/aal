use crate::array::Noun;
use phf::phf_map;

mod monads {
    use crate::array::Noun;
    use anyhow::Result;

    pub(crate) fn same(w: Noun) -> Result<Noun> {
        Ok(w)
    }
}

type MonadFn = fn(Noun) -> anyhow::Result<Noun>;

pub static MONADS: phf::Map<&'static str, MonadFn> = phf_map! {
    "]" => monads::same,
    "[" => monads::same,
};

mod dyads {
    use anyhow::{Context, Result};
    use crate::array::{IntegerElt, Noun};

    pub(crate) fn same_w(_: Noun, w: Noun) -> Result<Noun> {
        Ok(w)
    }

    pub(crate) fn same_a(a: Noun, _: Noun) -> Result<Noun> {
        Ok(a)
    }

    pub(crate) fn add(a: Noun, w: Noun) -> Result<Noun> {
        use crate::array::Array as Arr;
        use crate::array::ArrayOrAtom as AoA;
        use crate::array::Atom as At;
        use crate::array::MatchingNouns as MN;
        use crate::array::Noun as N;
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

type DyadFn = fn(Noun, Noun) -> anyhow::Result<Noun>;

pub static DYADS: phf::Map<&'static str, DyadFn> = phf_map! {
    "]" => dyads::same_w,
    "[" => dyads::same_a,
    "+" => dyads::add,
};
