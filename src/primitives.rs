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
            _ => {
                return Err(anyhow!(
                    "Incompatible argument: must be an atom or rank-1 integer array"
                ))
            }
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
    use crate::array::{GenericMatchingNouns, IntegerElt, Noun};
    use anyhow::{Context, Result};

    pub fn same_w(_: Noun, w: Noun) -> Result<Noun> {
        Ok(w)
    }

    pub fn same_a(a: Noun, _: Noun) -> Result<Noun> {
        Ok(a)
    }

    pub fn add(a: Noun, w: Noun) -> Result<Noun> {
        Noun::try_promote_pair(a, w)
            .context("failed to promote")?
            .dyad(
                |a, w| a as IntegerElt + w as IntegerElt,
                |a, w| a + w,
                |a, w| a + w,
            )
    }

    pub fn sub(a: Noun, w: Noun) -> Result<Noun> {
        Noun::try_promote_pair(a, w)
            .context("failed to promote")?
            .dyad(
                |a, w| a as IntegerElt - w as IntegerElt,
                |a, w| a - w,
                |a, w| a - w,
            )
    }

    pub fn mul(a: Noun, w: Noun) -> Result<Noun> {
        Noun::try_promote_pair(a, w)
            .context("failed to promote")?
            .dyad(|a, w| a && w, |a, w| a * w, |a, w| a * w)
    }

    pub fn and(a: Noun, w: Noun) -> Result<Noun> {
        GenericMatchingNouns::from((a.to_boolean(), w.to_boolean()))
            .dyad(|a, w| a && w)
            .map(Noun::from)
            .context("huh")
    }

    pub fn eq(a: Noun, w: Noun) -> Result<Noun> {
        Noun::try_promote_pair(a, w)
            .context("")?
            .dyad(|a, w| a == w, |a, w| a == w, |a, w| a == w)
    }
}

type DyadFn = fn(Noun, Noun) -> anyhow::Result<Noun>;

pub static DYADS: phf::Map<&'static str, DyadFn> = phf_map! {
    "]" => dyads::same_w,
    "[" => dyads::same_a,
    "+" => dyads::add,
    "-" => dyads::sub,
    "*" => dyads::mul,
    "*." => dyads::and,
    "=" => dyads::eq,
};
