use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::actors::{compare, intersects, is_contained, is_subset, is_superset};
use crate::error::FilsonResult;
use crate::types::Op;
use crate::utils::FalliableEntry;
use crate::{Appliable, DataNode, Extractable, FilsonError};

macro_rules! run_with_caching {
    // exists
    ($extractable:ident, $lhs: ident, $cache:ident) => {{
        let mut cache_borrow = $cache.borrow_mut();
        let entry = cache_borrow.entry($lhs);
        FalliableEntry::from(entry)
            .or_try_insert_with(|| $extractable.extract($lhs))
            .is_ok()
    }};

    // is_contained
    ($extractable:ident, $lhs: ident, $rhs: ident, $cache:ident, $fn:ident) => {{
        let mut cache_borrow = $cache.borrow_mut();
        let entry = cache_borrow.entry($lhs);
        let extracted =
            FalliableEntry::from(entry).or_try_insert_with(|| $extractable.extract($lhs))?;
        $fn(extracted, $rhs)
    }};

    // intersects, is_subset, is_superset
    ($extractable:ident, $lhs: ident, $rhs: ident, $cache:ident, $err_type:expr, $fn:ident) => {{
        let mut cache_borrow = $cache.borrow_mut();
        let entry = cache_borrow.entry($lhs);
        let extracted =
            FalliableEntry::from(entry).or_try_insert_with(|| $extractable.extract($lhs))?;
        extracted.error_on_not_collection_or_string($err_type)?;
        extracted.error_on_type_mismatch($rhs)?;
        $fn(extracted, $rhs)
    }};

    // compare
    (#[cfg($attr: meta)], $extractable:ident, $lhs: ident, $rhs: ident, $op:ident, $cache:ident, $err_type:expr, $fn:ident) => {{
        let mut cache_borrow = $cache.borrow_mut();
        let entry = cache_borrow.entry($lhs);
        let extracted =
            FalliableEntry::from(entry).or_try_insert_with(|| $extractable.extract($lhs))?;
        extracted.error_on_type_mismatch($rhs)?;
        #[cfg($attr)]
        if extracted.is_collection_type() & $op.is_ordering() {
            return Err($err_type);
        }
        $fn(extracted, $op, $rhs)
    }};
}

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
        let cache = Rc::new(RefCell::new(HashMap::new()));
        fn recursive_apply<'a, T: Extractable + 'a>(
            ast: &'a Ast<'a>,
            extractable: &'a T,
            cache: Rc<RefCell<HashMap<&'a str, DataNode<'a>>>>,
        ) -> FilsonResult<bool> {
            let res = match ast {
                Ast::And(lhs, rhs) => {
                    recursive_apply(lhs, extractable, cache.clone())?
                        & recursive_apply(rhs, extractable, cache.clone())?
                }
                Ast::Or(lhs, rhs) => {
                    recursive_apply(lhs, extractable, cache.clone())?
                        | recursive_apply(rhs, extractable, cache.clone())?
                }
                Ast::Xor(lhs, rhs) => {
                    recursive_apply(lhs, extractable, cache.clone())?
                        ^ recursive_apply(rhs, extractable, cache.clone())?
                }
                Ast::Not(inner) => !inner.apply(extractable)?,
                Ast::Compare { lhs, op, rhs } => {
                    run_with_caching!(
                        #[cfg(not(feature = "collection_ordering"))],
                        extractable,
                        lhs,
                        rhs,
                        op,
                        cache,
                        FilsonError::OrderingProhibitedError,
                        compare
                    )
                }
                Ast::Intersects { lhs, rhs } => {
                    run_with_caching!(
                        extractable,
                        lhs,
                        rhs,
                        cache,
                        FilsonError::IntersectsError,
                        intersects
                    )
                }
                Ast::IsContained { lhs, rhs } => {
                    run_with_caching!(extractable, lhs, rhs, cache, is_contained)
                }
                Ast::Exists { path } => {
                    run_with_caching!(extractable, path, cache)
                }
                Ast::IsSubset { lhs, rhs } => {
                    run_with_caching!(
                        extractable,
                        lhs,
                        rhs,
                        cache,
                        FilsonError::IsSubsetError,
                        is_subset
                    )
                }
                Ast::IsSuperset { lhs, rhs } => {
                    run_with_caching!(
                        extractable,
                        lhs,
                        rhs,
                        cache,
                        FilsonError::IsSupersetError,
                        is_superset
                    )
                }
            };
            Ok(res)
        }
        recursive_apply(self, v, cache.clone())
    }
}
