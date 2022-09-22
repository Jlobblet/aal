use crate::array::Noun;
use phf::phf_map;

mod monads {
    use crate::array::{Array, Atom, GenericArray, Noun};
    use anyhow::{anyhow, Result};

    pub fn same(w: Noun) -> Result<Noun> {
        Ok(w)
    }

    pub fn iota(w: Noun) -> Result<Noun> {
        let shape = match w {
            Noun::Atom(Atom::Integer(w)) => vec![w],
            Noun::Array(Array::Integer(w)) if w.rank() == 1 => w.raw_data().to_vec(),
            _ => return Err(anyhow!("Incompatible argument: must be an atom or rank-1 integer array"))
        };
        Ok(Noun::Array(Array::Integer(GenericArray::iota(&shape))))
    }
}

type MonadFn = fn(Noun) -> anyhow::Result<Noun>;

pub static MONADS: phf::Map<&'static str, MonadFn> = phf_map! {
    "]" => monads::same,
    "[" => monads::same,
    "i." => monads::iota,
};

mod dyads {
    use anyhow::{Context, Result};
    use crate::array::{IntegerElt, Noun};

    pub fn same_w(_: Noun, w: Noun) -> Result<Noun> {
        Ok(w)
    }

    pub fn same_a(a: Noun, _: Noun) -> Result<Noun> {
        Ok(a)
    }

    pub fn add(a: Noun, w: Noun) -> Result<Noun> {
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
