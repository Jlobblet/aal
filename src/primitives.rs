use crate::arrays::noun::Noun;
use phf::phf_map;

mod monads {
    use crate::arrays::array::Array;
    use crate::arrays::atom::Atom;
    use crate::arrays::generic_array::GenericArray;
    use crate::arrays::noun::Noun;
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
                    "Incompatible argument: must be an atom or rank-1 integer arrays"
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
    use crate::arrays::generic_matching_nouns::GenericMatchingNouns;
    use crate::arrays::noun::Noun;
    use crate::arrays::{DecimalElt, IntegerElt};
    use anyhow::{Context, Result};
    use crate::arrays::promote::Promote;

    pub fn same_w(_: Noun, w: Noun) -> Result<Noun> {
        Ok(w)
    }

    pub fn same_a(a: Noun, _: Noun) -> Result<Noun> {
        Ok(a)
    }

    pub fn add(a: Noun, w: Noun) -> Result<Noun> {
        Noun::try_promote_pair(a, w)
            .context("in dyadic + add")?
            .dyad(
                |a, w| a as IntegerElt + w as IntegerElt,
                |a, w| a + w,
                |a, w| a + w,
            )
    }

    pub fn sub(a: Noun, w: Noun) -> Result<Noun> {
        Noun::try_promote_pair(a, w)
            .context("in dyadic - sub")?
            .dyad(
                |a, w| a as IntegerElt - w as IntegerElt,
                |a, w| a - w,
                |a, w| a - w,
            )
    }

    pub fn mul(a: Noun, w: Noun) -> Result<Noun> {
        Noun::try_promote_pair(a, w)
            .context("in dyadic * mul")?
            .dyad(|a, w| a && w, |a, w| a * w, |a, w| a * w)
    }

    pub fn div(a: Noun, w: Noun) -> Result<Noun> {
        Noun::try_promote_pair(a, w)
            .context("in dyadic % div")?
            .dyad(
                |a, w| <bool as Promote<DecimalElt>>::promote(a) / <bool as Promote<DecimalElt>>::promote(w),
                |a, w| <IntegerElt as Promote<DecimalElt>>::promote(a) / <IntegerElt as Promote<DecimalElt>>::promote(w),
                |a, w| a / w
            )
    }

    pub fn and(a: Noun, w: Noun) -> Result<Noun> {
        GenericMatchingNouns::from((a.into_boolean(), w.into_boolean()))
            .dyad(|a, w| a && w)
            .context("in dyadic *. and")
            .map(Noun::from)
    }

    pub fn eq(a: Noun, w: Noun) -> Result<Noun> {
        Noun::try_promote_pair(a, w)
            .context("in dyadic = eq")?
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
    "%" => dyads::div,
    "*." => dyads::and,
    "=" => dyads::eq,
};
