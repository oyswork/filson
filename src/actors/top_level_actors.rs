use crate::error::FilsonResult;
use crate::types::Op;
use crate::{DataNode, Extractable, FilsonError};

use super::helpers::{intersects_logic, is_subset_logic};

macro_rules! error_on_not_collection_or_string {
    ($extracted_value: ident, $error_type: expr) => {
        if !$extracted_value.is_collection_type() & !$extracted_value.is_string_type() {
            return Err($error_type);
        }
    };
}

pub(crate) fn compare(
    data_bearer: &impl Extractable,
    path: &str,
    operation: &Op,
    value_to_compare_with: &DataNode,
) -> FilsonResult<bool> {
    let extracted_value = data_bearer.extract(path)?;
    extracted_value.error_on_type_mismatch(value_to_compare_with)?;
    #[cfg(not(feature = "collection_ordering"))]
    if extracted_value.is_collection_type() & operation.is_ordering() {
        return Err(FilsonError::OrderingProhibitedError);
    }
    Ok(match operation {
        Op::Eq => extracted_value == *value_to_compare_with,
        Op::Ne => extracted_value != *value_to_compare_with,
        Op::Gt => extracted_value > *value_to_compare_with,
        Op::Lt => extracted_value < *value_to_compare_with,
        Op::Gte => extracted_value >= *value_to_compare_with,
        Op::Lte => extracted_value <= *value_to_compare_with,
    })
}

pub(crate) fn exists(data_bearer: &impl Extractable, path: &str) -> bool {
    data_bearer.extract(path).is_ok()
}

pub(crate) fn intersects(
    data_bearer: &impl Extractable,
    path: &str,
    compound_or_str: &DataNode,
) -> FilsonResult<bool> {
    let extracted_value = data_bearer.extract(path)?;
    error_on_not_collection_or_string!(extracted_value, FilsonError::IntersectsError);
    extracted_value.error_on_type_mismatch(compound_or_str)?;
    Ok(intersects_logic(&extracted_value, compound_or_str))
}

pub(crate) fn is_contained(
    data_bearer: &impl Extractable,
    path: &str,
    compound: &DataNode,
) -> FilsonResult<bool> {
    let extracted_value = data_bearer.extract(path)?;
    match compound {
        DataNode::Set(s) => Ok(s.contains(&extracted_value)),
        DataNode::Array(arr) => Ok(arr.contains(&extracted_value)),
        DataNode::Map(m) => Ok(m.contains_key(&extracted_value)),
        DataNode::I64(_)
        | DataNode::F64(_)
        | DataNode::Str(_)
        | DataNode::Bool(_)
        | DataNode::Null => unreachable!(),
    }
}

pub(crate) fn is_subset(
    data_bearer: &impl Extractable,
    path: &str,
    compound_or_str: &DataNode,
) -> FilsonResult<bool> {
    let extracted_value = data_bearer.extract(path)?;
    error_on_not_collection_or_string!(extracted_value, FilsonError::IsSubsetError);
    extracted_value.error_on_type_mismatch(compound_or_str)?;
    Ok(is_subset_logic(&extracted_value, compound_or_str))
}

pub(crate) fn is_superset(
    data_bearer: &impl Extractable,
    path: &str,
    compound_or_str: &DataNode,
) -> FilsonResult<bool> {
    let extracted_value = data_bearer.extract(path)?;
    error_on_not_collection_or_string!(extracted_value, FilsonError::IsSupersetError);
    extracted_value.error_on_type_mismatch(compound_or_str)?;
    Ok(is_subset_logic(compound_or_str, &extracted_value))
}
