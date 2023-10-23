use crate::types::Op;
use crate::DataNode;

use super::helpers::{intersects_logic, is_subset_logic};

pub(crate) fn compare(
    extracted_value: &DataNode,
    operation: &Op,
    value_to_compare_with: &DataNode,
) -> bool {
    match operation {
        Op::Eq => extracted_value == value_to_compare_with,
        Op::Ne => extracted_value != value_to_compare_with,
        Op::Gt => extracted_value > value_to_compare_with,
        Op::Lt => extracted_value < value_to_compare_with,
        Op::Gte => extracted_value >= value_to_compare_with,
        Op::Lte => extracted_value <= value_to_compare_with,
    }
}

pub(crate) fn intersects(extracted_value: &DataNode, compound_or_str: &DataNode) -> bool {
    intersects_logic(extracted_value, compound_or_str)
}

pub(crate) fn is_contained(extracted_value: &DataNode, compound: &DataNode) -> bool {
    match compound {
        DataNode::Set(s) => s.contains(extracted_value),
        DataNode::Array(arr) => arr.contains(extracted_value),
        DataNode::Map(m) => m.contains_key(extracted_value),
        DataNode::I64(_)
        | DataNode::F64(_)
        | DataNode::Str(_)
        | DataNode::Bool(_)
        | DataNode::Null => unreachable!(),
    }
}

pub(crate) fn is_subset(extracted_value: &DataNode, compound_or_str: &DataNode) -> bool {
    is_subset_logic(extracted_value, compound_or_str)
}

pub(crate) fn is_superset(extracted_value: &DataNode, compound_or_str: &DataNode) -> bool {
    is_subset_logic(compound_or_str, extracted_value)
}
