use crate::actors::traits::{Compare, Contains, Intersects, IsSubset, IsSuperset};
use crate::error::FilsonResult;
use crate::types::Op;
use crate::{Appliable, DataNode, Extractable, FilsonError};
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "extraction_caching")] {
        use crate::ast::extraction_utils_cached as extraction_utils;
        use fxhash::FxHashMap;
        use std::ptr::{addr_of_mut, NonNull};
    } else {
        use crate::ast::extraction_utils_uncached as extraction_utils;
    }
}
use extraction_utils::{get_extractable, CacheType};

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
        cfg_if! {
            if #[cfg(feature = "extraction_caching")] {
                let mut cache_map = FxHashMap::default();
                let cache =
                    Some(NonNull::new(addr_of_mut!(cache_map)).ok_or(FilsonError::CacheCreationError)?);
            } else {
                let cache: Option<CacheType> = None;
            }
        }

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
                    extracted.compare(*op, rhs)
                }
                Ast::Intersects { lhs, rhs } => {
                    let extracted = get_extractable(lhs, extractable, cache)?;
                    extracted.error_on_not_collection_or_string(FilsonError::IntersectsError)?;
                    extracted.error_on_type_mismatch(rhs)?;
                    extracted.intersects(rhs)
                }
                Ast::IsContained { lhs, rhs } => {
                    let extracted = get_extractable(lhs, extractable, cache)?;
                    rhs.contains(&extracted)
                }
                Ast::Exists { path } => {
                    let extracted = get_extractable(path, extractable, cache);
                    extracted.is_ok()
                }
                Ast::IsSubset { lhs, rhs } => {
                    let extracted = get_extractable(lhs, extractable, cache)?;
                    extracted.error_on_not_collection_or_string(FilsonError::IsSubsetError)?;
                    extracted.error_on_type_mismatch(rhs)?;
                    extracted.is_subset(rhs)
                }
                Ast::IsSuperset { lhs, rhs } => {
                    let extracted = get_extractable(lhs, extractable, cache)?;
                    extracted.error_on_not_collection_or_string(FilsonError::IsSupersetError)?;
                    extracted.error_on_type_mismatch(rhs)?;
                    extracted.is_superset(rhs)
                }
            };
            Ok(res)
        }
        recursive_apply(self, extractable, cache)
    }
}
