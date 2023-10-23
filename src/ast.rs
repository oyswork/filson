use crate::actors::{compare, intersects, is_contained, is_subset, is_superset};
use crate::error::FilsonResult;
use crate::types::Op;
use crate::{Appliable, DataNode, Extractable, FilsonError};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Ast<'a> {
    And(Box<Ast<'a>>, Box<Ast<'a>>),
    Or(Box<Ast<'a>>, Box<Ast<'a>>),
    Xor(Box<Ast<'a>>, Box<Ast<'a>>),
    Not(Box<Ast<'a>>),
    Compare {
        lhs: &'a str,
        op: Op,
        rhs: DataNode<'a>,
    },
    Intersects {
        lhs: &'a str,
        rhs: DataNode<'a>,
    },
    IsContained {
        lhs: &'a str,
        rhs: DataNode<'a>,
    },
    Exists {
        path: &'a str,
    },
    IsSubset {
        lhs: &'a str,
        rhs: DataNode<'a>,
    },
    IsSuperset {
        lhs: &'a str,
        rhs: DataNode<'a>,
    },
}

impl Appliable for Ast<'_> {
    fn apply<T: Extractable>(&self, v: &T) -> FilsonResult<bool> {
        let res = match self {
            Ast::And(lhs, rhs) => lhs.apply(v)? & rhs.apply(v)?,
            Ast::Or(lhs, rhs) => lhs.apply(v)? | rhs.apply(v)?,
            Ast::Xor(lsh, rhs) => lsh.apply(v)? ^ rhs.apply(v)?,
            Ast::Not(inner) => !inner.apply(v)?,
            Ast::Compare { lhs, op, rhs } => {
                let extracted = try_extract(lhs, v)?;
                extracted.error_on_type_mismatch(rhs)?;
                #[cfg(not(feature = "collection_ordering"))]
                if extracted.is_collection_type() & op.is_ordering() {
                    return Err(FilsonError::OrderingProhibitedError);
                }
                compare(&extracted, op, rhs)
            }
            Ast::Intersects { lhs, rhs } => {
                let extracted = try_extract(lhs, v)?;
                extracted.error_on_not_collection_or_string(FilsonError::IntersectsError)?;
                extracted.error_on_type_mismatch(rhs)?;
                intersects(&extracted, rhs)
            }
            Ast::IsContained { lhs, rhs } => {
                let extracted = try_extract(lhs, v)?;
                is_contained(&extracted, rhs)
            }
            Ast::Exists { path } => try_extract(path, v).is_ok(),
            Ast::IsSubset { lhs, rhs } => {
                let extracted = try_extract(lhs, v)?;
                extracted.error_on_not_collection_or_string(FilsonError::IsSubsetError)?;
                extracted.error_on_type_mismatch(rhs)?;
                is_subset(&extracted, rhs)
            }
            Ast::IsSuperset { lhs, rhs } => {
                let extracted = try_extract(lhs, v)?;
                extracted.error_on_not_collection_or_string(FilsonError::IsSupersetError)?;
                extracted.error_on_type_mismatch(rhs)?;
                is_superset(&extracted, rhs)
            }
        };
        Ok(res)
    }
}

fn try_extract<'a>(
    path: &'a str,
    data_brearer: &'a impl Extractable,
) -> FilsonResult<DataNode<'a>> {
    data_brearer.extract(path)
}
