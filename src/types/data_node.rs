use std::{
    collections::{BTreeMap, BTreeSet},
    mem::discriminant,
};

use crate::{
    error::FilsonResult,
    parser::{parse_float, parse_int, Rule},
};

use ordered_float::OrderedFloat;
use pest::iterators::Pair;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Hash)]
pub enum DataNode<'a> {
    Map(BTreeMap<DataNode<'a>, DataNode<'a>>),
    Set(BTreeSet<DataNode<'a>>),
    Array(Vec<DataNode<'a>>),
    I64(i64),
    F64(OrderedFloat<f64>),
    Str(&'a str),
    Bool(bool),
    Null,
}

impl<'a> From<&'a str> for DataNode<'a> {
    fn from(s: &'a str) -> Self {
        Self::Str(s)
    }
}

impl From<bool> for DataNode<'_> {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<i64> for DataNode<'_> {
    fn from(i: i64) -> Self {
        Self::I64(i)
    }
}

impl From<f64> for DataNode<'_> {
    fn from(f: f64) -> Self {
        Self::F64(OrderedFloat(f))
    }
}

impl<'a, T> From<Option<T>> for DataNode<'a>
where
    T: Into<DataNode<'a>>,
{
    fn from(value: Option<T>) -> Self {
        if let Some(t) = value {
            t.into()
        } else {
            Self::Null
        }
    }
}

impl<'a> From<BTreeSet<DataNode<'a>>> for DataNode<'a> {
    fn from(set: BTreeSet<DataNode<'a>>) -> Self {
        Self::Set(set)
    }
}

impl<'a> From<Vec<DataNode<'a>>> for DataNode<'a> {
    fn from(arr: Vec<DataNode<'a>>) -> Self {
        Self::Array(arr)
    }
}

impl<'a> From<BTreeMap<DataNode<'a>, DataNode<'a>>> for DataNode<'a> {
    fn from(map: BTreeMap<DataNode<'a>, DataNode<'a>>) -> Self {
        Self::Map(map)
    }
}

//  unwraping is ok because parser won't allow for unsupported formats
impl<'a> From<Pair<'a, Rule>> for DataNode<'a> {
    fn from(pair: Pair<'a, Rule>) -> Self {
        match pair.as_rule() {
            Rule::boolean => (pair.as_str() == "true").into(),

            Rule::null => Self::Null,

            Rule::string => {
                let chars = pair.into_inner().next().unwrap();
                chars.as_str().into()
            }

            Rule::integer => parse_int(pair.as_str()).unwrap().into(),

            Rule::float => parse_float(pair.as_str()).unwrap().into(),

            Rule::array => pair
                .into_inner()
                .map(DataNode::from)
                .collect::<Vec<_>>()
                .into(),

            Rule::set => pair
                .into_inner()
                .map(DataNode::from)
                .collect::<BTreeSet<_>>()
                .into(),

            Rule::map => pair
                .into_inner()
                .map(|map_pair| {
                    let mut inner_map_pair = map_pair.into_inner();
                    let key = inner_map_pair.next().unwrap();
                    let value = inner_map_pair.next().unwrap();
                    (DataNode::from(key), DataNode::from(value))
                })
                .collect::<BTreeMap<_, _>>()
                .into(),

            _ => unreachable!(),
        }
    }
}

impl DataNode<'_> {
    pub(crate) fn is_collection_type(&self) -> bool {
        matches!(self, DataNode::Map(_) | DataNode::Set(_) | DataNode::Array(_))
    }

    pub(crate) fn is_string_type(&self) -> bool {
        matches!(self, DataNode::Str(_))
    }

    #[inline]
    pub(crate) fn error_on_type_mismatch(&self, other: &Self) -> FilsonResult<()> {
        if discriminant(self) != discriminant(other) {
            return Err(crate::FilsonError::TypeError);
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fn_name() {
        dbg!(DataNode::Array(vec![1.into()]) == DataNode::Null);
    }
}
