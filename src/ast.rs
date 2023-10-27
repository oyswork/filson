use std::collections::HashMap;

use crate::actors::{compare, intersects, is_contained, is_subset, is_superset};
use crate::error::FilsonResult;
use crate::types::Op;
use crate::utils::FalliableEntry;
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

#[inline]
fn get_extractable<'a>(
    cache: *mut HashMap<&'a str, DataNode<'a>>,
    path: &'a str,
    extractable: &'a impl Extractable,
) -> FilsonResult<&'a mut DataNode<'a>> {
    unsafe {
        FalliableEntry::from((*cache).entry(path)).or_try_insert_with(|| extractable.extract(path))
    }
}

impl Appliable for Ast<'_> {
    fn apply<T: Extractable>(&self, v: &T) -> FilsonResult<bool> {
        let mut cache = HashMap::new();
        fn recursive_apply<'a, T: Extractable + 'a>(
            ast: &'a Ast<'a>,
            extractable: &'a T,
            cache: *mut HashMap<&'a str, DataNode<'a>>,
        ) -> FilsonResult<bool> {
            let res = match ast {
                Ast::And(lhs, rhs) => {
                    recursive_apply(lhs, extractable, cache)?
                        & recursive_apply(rhs, extractable, cache)?
                }
                Ast::Or(lhs, rhs) => {
                    recursive_apply(lhs, extractable, cache)?
                        | recursive_apply(rhs, extractable, cache)?
                }
                Ast::Xor(lhs, rhs) => {
                    recursive_apply(lhs, extractable, cache)?
                        ^ recursive_apply(rhs, extractable, cache)?
                }
                Ast::Not(inner) => !inner.apply(extractable)?,
                Ast::Compare { lhs, op, rhs } => {
                    let extracted = get_extractable(cache, lhs, extractable)?;
                    extracted.error_on_type_mismatch(rhs)?;
                    #[cfg(not(feature = "collections_ordering"))]
                    if extracted.is_collection_type() & op.is_ordering() {
                        return Err(FilsonError::OrderingProhibitedError);
                    }
                    compare(extracted, op, rhs)
                }
                Ast::Intersects { lhs, rhs } => {
                    let extracted = get_extractable(cache, lhs, extractable)?;
                    extracted.error_on_not_collection_or_string(FilsonError::IntersectsError)?;
                    extracted.error_on_type_mismatch(rhs)?;
                    intersects(extracted, rhs)
                }
                Ast::IsContained { lhs, rhs } => {
                    let extracted = get_extractable(cache, lhs, extractable)?;
                    is_contained(extracted, rhs)
                }
                Ast::Exists { path } => {
                    let extracted = get_extractable(cache, path, extractable);
                    extracted.is_ok()
                }
                Ast::IsSubset { lhs, rhs } => {
                    let extracted = get_extractable(cache, lhs, extractable)?;
                    extracted.error_on_not_collection_or_string(FilsonError::IsSubsetError)?;
                    extracted.error_on_type_mismatch(rhs)?;
                    is_subset(extracted, rhs)
                }
                Ast::IsSuperset { lhs, rhs } => {
                    let extracted = get_extractable(cache, lhs, extractable)?;
                    extracted.error_on_not_collection_or_string(FilsonError::IsSupersetError)?;
                    extracted.error_on_type_mismatch(rhs)?;
                    is_superset(extracted, rhs)
                }
            };
            Ok(res)
        }
        recursive_apply(self, v, &mut cache as *mut HashMap<&str, DataNode<'_>>)
    }
}
