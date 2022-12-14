use crate::arrays::generic_array::GenericArray;
use crate::arrays::{DecimalElt, IntegerElt};

#[derive(Debug, Clone)]
pub enum Array {
    Boolean(GenericArray<bool>),
    Integer(GenericArray<IntegerElt>),
    Decimal(GenericArray<DecimalElt>),
}

impl From<GenericArray<bool>> for Array {
    fn from(w: GenericArray<bool>) -> Self {
        Self::Boolean(w)
    }
}

impl From<GenericArray<IntegerElt>> for Array {
    fn from(w: GenericArray<IntegerElt>) -> Self {
        Self::Integer(w)
    }
}

impl From<GenericArray<DecimalElt>> for Array {
    fn from(w: GenericArray<DecimalElt>) -> Self {
        Self::Decimal(w)
    }
}

impl Array {
    pub fn shape(&self) -> &[usize] {
        use Array::*;
        match self {
            Boolean(b) => b.shape(),
            Integer(i) => i.shape(),
            Decimal(d) => d.shape(),
        }
    }

    pub fn rank(&self) -> usize {
        use Array::*;
        match self {
            Boolean(b) => b.rank(),
            Integer(i) => i.rank(),
            Decimal(d) => d.rank(),
        }
    }
}
