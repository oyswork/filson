use crate::actors::{compare, exists, intersects, is_contained, is_subset, is_superset};
use crate::error::FilsonResult;
use crate::types::Op;
use crate::{Appliable, DataNode, Extractable};

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
            Ast::Compare { lhs, op, rhs } => compare(v, lhs, op, rhs)?,
            Ast::Intersects { lhs, rhs } => intersects(v, lhs, rhs)?,
            Ast::IsContained { lhs, rhs } => is_contained(v, lhs, rhs)?,
            Ast::Exists { path } => exists(v, path),
            Ast::IsSubset { lhs, rhs } => is_subset(v, lhs, rhs)?,
            Ast::IsSuperset { lhs, rhs } => is_superset(v, lhs, rhs)?,
        };
        Ok(res)
    }
}
