use crate::types::Op;
use crate::DataNode;

use super::helpers::{array_is_subset, btreemap_intersects, map_is_subset, str_array_intersects};

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
    match (extracted_value, compound_or_str) {
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
    match (extracted_value, compound_or_str) {
        (DataNode::Set(left_set), DataNode::Set(right_set)) => left_set.is_subset(right_set),
        (DataNode::Array(left_arr), DataNode::Array(right_arr)) => {
            array_is_subset(left_arr, right_arr)
        }
        (DataNode::Map(left_map), DataNode::Map(right_map)) => map_is_subset(left_map, right_map),
        (DataNode::Str(left_str), DataNode::Str(right_str)) => right_str.contains(left_str),
        _ => unreachable!(),
    }
}

pub(crate) fn is_superset(extracted_value: &DataNode, compound_or_str: &DataNode) -> bool {
    match (compound_or_str, extracted_value) {
        (DataNode::Set(left_set), DataNode::Set(right_set)) => left_set.is_subset(right_set),
        (DataNode::Array(left_arr), DataNode::Array(right_arr)) => {
            array_is_subset(left_arr, right_arr)
        }
        (DataNode::Map(left_map), DataNode::Map(right_map)) => map_is_subset(left_map, right_map),
        (DataNode::Str(left_str), DataNode::Str(right_str)) => right_str.contains(left_str),
        _ => unreachable!(),
    }
}
