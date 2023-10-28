#[cfg(feature = "extraction_caching")]
use fxhash::FxHashMap;

use crate::actors::{compare, intersects, is_contained, is_subset, is_superset};
use crate::ast::extraction_utils::{get_extractable, CacheType};
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
    fn apply<T: Extractable>(&self, extractable: &T) -> FilsonResult<bool> {
        #[cfg(feature = "extraction_caching")]
        let mut cache_map = FxHashMap::default();
        #[cfg(feature = "extraction_caching")]
        let cache = Some(&mut cache_map as CacheType);
        #[cfg(not(feature = "extraction_caching"))]
        let cache: Option<CacheType> = None;

        fn recursive_apply<'a, T: Extractable + 'a>(
            ast: &'a Ast<'a>,
            extractable: &'a T,
            cache: Option<CacheType<'a>>,
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
                    let extracted = get_extractable(lhs, extractable, cache)?;
                    extracted.error_on_type_mismatch(rhs)?;
                    #[cfg(not(feature = "collections_ordering"))]
                    if extracted.is_collection_type() & op.is_ordering() {
                        return Err(FilsonError::OrderingProhibitedError);
                    }
                    compare(&extracted, op, rhs)
                }
                Ast::Intersects { lhs, rhs } => {
                    let extracted = get_extractable(lhs, extractable, cache)?;
                    extracted.error_on_not_collection_or_string(FilsonError::IntersectsError)?;
                    extracted.error_on_type_mismatch(rhs)?;
                    intersects(&extracted, rhs)
                }
                Ast::IsContained { lhs, rhs } => {
                    let extracted = get_extractable(lhs, extractable, cache)?;
                    is_contained(&extracted, rhs)
                }
                Ast::Exists { path } => {
                    let extracted = get_extractable(path, extractable, cache);
                    extracted.is_ok()
                }
                Ast::IsSubset { lhs, rhs } => {
                    let extracted = get_extractable(lhs, extractable, cache)?;
                    extracted.error_on_not_collection_or_string(FilsonError::IsSubsetError)?;
                    extracted.error_on_type_mismatch(rhs)?;
                    is_subset(&extracted, rhs)
                }
                Ast::IsSuperset { lhs, rhs } => {
                    let extracted = get_extractable(lhs, extractable, cache)?;
                    extracted.error_on_not_collection_or_string(FilsonError::IsSupersetError)?;
                    extracted.error_on_type_mismatch(rhs)?;
                    is_superset(&extracted, rhs)
                }
            };
            Ok(res)
        }
        recursive_apply(self, extractable, cache)
    }
}
