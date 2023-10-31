use crate::{
    actors::{
        helpers::{array_is_subset, btreemap_intersects, map_is_subset, str_array_intersects},
        traits::definitions::{Compare, Contains, Intersects, IsSubset, IsSuperset},
    },
    types::{DataNode, Op},
};

impl Contains for DataNode<'_> {
    fn contains(&self, other: &Self) -> bool {
        match self {
            DataNode::Set(s) => s.contains(other),
            DataNode::Array(arr) => arr.contains(other),
            DataNode::Map(m) => m.contains_key(other),
            DataNode::I64(_)
            | DataNode::F64(_)
            | DataNode::Str(_)
            | DataNode::Bool(_)
            | DataNode::Null => unreachable!(),
        }
    }
}

impl IsSubset for DataNode<'_> {
    fn is_subset(&self, other: &Self) -> bool {
        match (self, other) {
            (DataNode::Set(left_set), DataNode::Set(right_set)) => left_set.is_subset(right_set),
            (DataNode::Array(left_arr), DataNode::Array(right_arr)) => {
                array_is_subset(left_arr, right_arr)
            }
            (DataNode::Map(left_map), DataNode::Map(right_map)) => {
                map_is_subset(left_map, right_map)
            }
            (DataNode::Str(left_str), DataNode::Str(right_str)) => right_str.contains(left_str),
            _ => unreachable!(),
        }
    }
}

impl<T: IsSubset> IsSuperset for T {
    fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }
}

impl Intersects for DataNode<'_> {
    fn intersects(&self, other: &Self) -> bool {
        match (self, other) {
            (DataNode::Set(left_set), DataNode::Set(right_set)) => {
                left_set.intersection(right_set).next().is_some()
            }
            (DataNode::Array(left_arr), DataNode::Array(right_arr)) => {
                str_array_intersects(left_arr.iter(), right_arr.iter())
            }
            (DataNode::Map(left_map), DataNode::Map(right_map)) => {
                btreemap_intersects(left_map, right_map)
            }
            (DataNode::Str(left_str), DataNode::Str(right_str)) => {
                str_array_intersects(left_str.bytes(), right_str.bytes())
            }
            _ => unreachable!(),
        }
    }
}

impl Compare for DataNode<'_> {
    fn compare(&self, operation: Op, other: &Self) -> bool {
        match operation {
            Op::Eq => self == other,
            Op::Ne => self != other,
            Op::Gt => self > other,
            Op::Lt => self < other,
            Op::Gte => self >= other,
            Op::Lte => self <= other,
        }
    }
}
