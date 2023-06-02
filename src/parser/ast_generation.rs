use pest::{iterators::Pair, Parser};

use crate::{ast::Ast, error::FilsonResult};

#[derive(pest_derive::Parser)]
#[grammar = "filson_grammar.pest"]
pub(crate) struct FilsonParser;

pub(crate) fn get_ast<'a>(inp: &'a str) -> FilsonResult<Ast<'a>> {
    let pair: Pair<'a, Rule> = match FilsonParser::parse(Rule::expression, inp) {
        Ok(mut pairs) => pairs.next().unwrap(),
        Err(pest_err) => Err(Box::new(pest_err))?,
    };

    fn get_ast_recursively(pair: Pair<Rule>) -> Ast {
        match pair.as_rule() {
            Rule::compare => {
                let mut inner = pair.into_inner();
                let pointer = inner.next().unwrap().into_inner().next().unwrap();
                let op = inner.next().unwrap();
                let comparee = inner.next().unwrap();
                // TODO type check and split into compare eq and compare ord in here
                Ast::Compare {
                    lhs: pointer.as_str(),
                    op: op.as_str().into(),
                    rhs: comparee.into(),
                }
            }

            Rule::intersects => {
                let mut inner = pair.into_inner();
                let pointer = inner.next().unwrap().into_inner().next().unwrap();
                let compound_or_str = inner.next().unwrap();
                Ast::Intersects {
                    lhs: pointer.as_str(),
                    rhs: compound_or_str.into(),
                }
            }

            Rule::is_contained => {
                let mut inner = pair.into_inner();
                let pointer = inner.next().unwrap().into_inner().next().unwrap();
                let compound = inner.next().unwrap();
                Ast::IsContained {
                    lhs: pointer.as_str(),
                    rhs: compound.into(),
                }
            }

            Rule::exists => {
                let mut inner = pair.into_inner();
                let pointer = inner.next().unwrap().into_inner().next().unwrap();
                Ast::Exists {
                    path: pointer.as_str(),
                }
            }

            Rule::is_subset => {
                let mut inner = pair.into_inner();
                let pointer = inner.next().unwrap().into_inner().next().unwrap();
                let compound_or_str = inner.next().unwrap();
                Ast::IsSubset {
                    lhs: pointer.as_str(),
                    rhs: compound_or_str.into(),
                }
            }

            Rule::is_superset => {
                let mut inner = pair.into_inner();
                let pointer = inner.next().unwrap().into_inner().next().unwrap();
                let compound_or_str = inner.next().unwrap();
                Ast::IsSuperset {
                    lhs: pointer.as_str(),
                    rhs: compound_or_str.into(),
                }
            }

            Rule::binary_operation => {
                let mut inner = pair.into_inner();
                let identifier = inner.next().unwrap();
                let body = inner.next().unwrap();
                let mut body_inner = body.into_inner();
                let left = body_inner.next().unwrap();
                let right = body_inner.next().unwrap();
                match identifier.as_str() {
                    "and" => Ast::And(
                        Box::new(get_ast_recursively(left)),
                        Box::new(get_ast_recursively(right)),
                    ),
                    "or" => Ast::Or(
                        Box::new(get_ast_recursively(left)),
                        Box::new(get_ast_recursively(right)),
                    ),
                    "xor" => Ast::Xor(
                        Box::new(get_ast_recursively(left)),
                        Box::new(get_ast_recursively(right)),
                    ),
                    _ => unreachable!(),
                }
            }

            Rule::not => {
                let mut inner = pair.into_inner();
                let inner_expr = inner.next().unwrap();
                Ast::Not(Box::new(get_ast_recursively(inner_expr)))
            }
            _ => unreachable!(),
        }
    }

    Ok(get_ast_recursively(pair))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;
    use crate::{types::Op, DataNode};

    #[test]
    fn parse_not() {
        assert_eq!(
            get_ast(r#"!compare("/id" == null)"#).unwrap(),
            Ast::Not(Box::new(Ast::Compare {
                lhs: "/id",
                op: Op::Eq,
                rhs: DataNode::Null
            }))
        );
    }

    #[test]
    fn parse_compare() {
        assert_eq!(
            get_ast(r#"compare("/id" == null)"#).unwrap(),
            Ast::Compare {
                lhs: "/id",
                op: Op::Eq,
                rhs: DataNode::Null
            }
        );
    }

    #[test]
    fn parse_intersects() {
        assert_eq!(
            get_ast(r#"intersects("/id" [1])"#).unwrap(),
            Ast::Intersects {
                lhs: "/id",
                rhs: vec![1.into()].into()
            }
        );
        assert_eq!(
            get_ast(r#"intersects("/id" {1})"#).unwrap(),
            Ast::Intersects {
                lhs: "/id",
                rhs: BTreeSet::from_iter(vec![1.into()]).into()
            }
        );
        assert_eq!(
            get_ast(r#"intersects("/id" <1:1>)"#).unwrap(),
            Ast::Intersects {
                lhs: "/id",
                rhs: BTreeMap::from_iter(vec![(1.into(), 1.into())]).into()
            }
        );
        assert_eq!(
            get_ast(r#"intersects("/id" "karl")"#).unwrap(),
            Ast::Intersects {
                lhs: "/id",
                rhs: "karl".into()
            }
        );
    }

    #[test]
    fn parse_is_contained() {
        assert_eq!(
            get_ast("is_contained(\"/id\" [1])").unwrap(),
            Ast::IsContained {
                lhs: "/id",
                rhs: vec![1.into()].into(),
            }
        );
        assert_eq!(
            get_ast("is_contained(\"/id\" {1})").unwrap(),
            Ast::IsContained {
                lhs: "/id",
                rhs: BTreeSet::from_iter(vec![1.into()]).into(),
            }
        );
        assert_eq!(
            get_ast("is_contained(\"/id\" <1:1>)").unwrap(),
            Ast::IsContained {
                lhs: "/id",
                rhs: BTreeMap::from_iter(vec![(1.into(), 1.into())]).into(),
            }
        );
    }

    #[test]
    fn parse_exists() {
        assert_eq!(
            get_ast(r#"exists("/id")"#).unwrap(),
            Ast::Exists { path: "/id" }
        )
    }

    #[test]
    fn parse_is_superset() {
        assert_eq!(
            get_ast(r#"is_superset("/id" [1])"#).unwrap(),
            Ast::IsSuperset {
                lhs: "/id",
                rhs: vec![1.into()].into(),
            }
        );
        assert_eq!(
            get_ast(r#"is_superset("/id" {1})"#).unwrap(),
            Ast::IsSuperset {
                lhs: "/id",
                rhs: BTreeSet::from_iter(vec![1.into()]).into(),
            }
        );
        assert_eq!(
            get_ast(r#"is_superset("/id" <1:1>)"#).unwrap(),
            Ast::IsSuperset {
                lhs: "/id",
                rhs: BTreeMap::from_iter(vec![(1.into(), 1.into())]).into(),
            }
        );
        assert_eq!(
            get_ast(r#"is_superset("/id" "karl")"#).unwrap(),
            Ast::IsSuperset {
                lhs: "/id",
                rhs: "karl".into()
            }
        );
    }

    #[test]
    fn parse_is_subset() {
        assert_eq!(
            get_ast(r#"is_subset("/id" [1])"#).unwrap(),
            Ast::IsSubset {
                lhs: "/id",
                rhs: vec![1.into()].into(),
            }
        );
        assert_eq!(
            get_ast(r#"is_subset("/id" {1})"#).unwrap(),
            Ast::IsSubset {
                lhs: "/id",
                rhs: BTreeSet::from_iter(vec![1.into()]).into(),
            }
        );
        assert_eq!(
            get_ast(r#"is_subset("/id" <1:1>)"#).unwrap(),
            Ast::IsSubset {
                lhs: "/id",
                rhs: BTreeMap::from_iter(vec![(1.into(), 1.into())]).into(),
            }
        );
        assert_eq!(
            get_ast(r#"is_subset("/id" "karl")"#).unwrap(),
            Ast::IsSubset {
                lhs: "/id",
                rhs: "karl".into()
            }
        );
    }

    #[test]
    fn parse_binary_and_condition_no_nesting() {
        assert_eq!(
            get_ast(r#"and(exists("/id"), compare("/id" == null))"#).unwrap(),
            Ast::And(
                Box::new(Ast::Exists { path: "/id" }),
                Box::new(Ast::Compare {
                    lhs: "/id",
                    op: Op::Eq,
                    rhs: DataNode::Null
                })
            )
        )
    }

    #[test]
    fn parse_binary_and_or_xor_not_conditions_multiple_nesting() {
        assert_eq!(
            get_ast(r#"xor(and(!exists("/id"), compare("/id" == null)), !or(exists("/id"), compare("/id" == null)))"#)
                .unwrap(),
            Ast::Xor(
                Box::new(Ast::And(
                    Box::new(
                        Ast::Not(
                            Box::new(
                                Ast::Exists{
                                    path: "/id"
                                }
                            )
                        )
                    ),
                    Box::new(Ast::Compare {
                        lhs:"/id",
                        op: Op::Eq,
                        rhs: DataNode::Null
                        }
                    )
                )
            ),
                Box::new(
                    Ast::Not(
                        Box::new(
                            Ast::Or(
                                Box::new(
                                    Ast::Exists{
                                        path: "/id"
                                    }
                                ),
                                Box::new(
                                    Ast::Compare{
                                        lhs: "/id",
                                        op:Op::Eq,
                                        rhs: DataNode::Null
                                    }
                                )
                            )
                        )
                    )
                )
            )
        )
    }
}
