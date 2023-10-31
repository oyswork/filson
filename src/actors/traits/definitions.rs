use crate::types::Op;

pub(crate) trait Contains {
    fn contains(&self, other: &Self) -> bool;
}

pub(crate) trait IsSubset {
    fn is_subset(&self, other: &Self) -> bool;
}

pub(crate) trait IsSuperset: IsSubset {
    fn is_superset(&self, other: &Self) -> bool;
}

pub(crate) trait Intersects {
    fn intersects(&self, other: &Self) -> bool;
}

pub(crate) trait Compare {
    fn compare(&self, op: Op, other: &Self) -> bool;
}
