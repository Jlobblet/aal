use crate::arrays::{DecimalElt, IntegerElt};

#[derive(Debug, Copy, Clone)]
pub enum Atom {
    Boolean(bool),
    Integer(IntegerElt),
    Decimal(DecimalElt),
}

impl From<bool> for Atom {
    fn from(w: bool) -> Self {
        Self::Boolean(w)
    }
}

impl From<IntegerElt> for Atom {
    fn from(w: IntegerElt) -> Self {
        Self::Integer(w)
    }
}

impl From<DecimalElt> for Atom {
    fn from(w: DecimalElt) -> Self {
        Self::Decimal(w)
    }
}
