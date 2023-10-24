use crate::{ast::Ast, error::FilsonResult};
use pest::{iterators::Pair, Parser};

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
mod test_primitive_types_parsing {
    use crate::parser::ast_generation::{FilsonParser, Rule};
    use pest::{consumes_to, fails_with, parses_to};

    const VALID_INTS: [&str; 4] = ["1", "+1", "-1", "1_2"];
    const INVALID_INTS: [&str; 2] = ["_1", "1_"];
    const VALID_FLOATS: [&str; 11] = [
        "1.2", "+1.2", "-1.2", ".1", "1_2.34", "12.3_4", "1.0e1", "1.0E1", "1.0e+1", "1.0e-1",
        "1.0e1_0",
    ];
    const INVALID_FLOATS: [&str; 8] = ["1.", "_1.0", "1_.0", "1._0", "1.0_", "1.E1", ".e1", "1.e"];

    #[test]
    fn parse_integer() {
        for i in VALID_INTS {
            parses_to! {
                parser: FilsonParser,
                input: i,
                rule: Rule::integer,
                tokens: [
                    integer(0, i.len())
                ]
            };
        }
        for i in INVALID_INTS {
            fails_with! {
                parser: FilsonParser,
                input: i,
                rule: Rule::integer,
                positives: [Rule::integer],
                negatives: [],
                pos: 0
            };
        }
    }

    #[test]
    fn parse_float() {
        for i in VALID_FLOATS {
            parses_to! {
                parser: FilsonParser,
                input: i,
                rule: Rule::float,
                tokens: [
                    float(0, i.len())
                ]
            };
        }
        for i in INVALID_FLOATS {
            fails_with! {
                parser: FilsonParser,
                input: i,
                rule: Rule::float,
                positives: [Rule::float],
                negatives: [],
                pos: 0
            };
        }
        // need to test these 2 in scope of compare and/or array/set
        // fails_with! {
        //     parser: FilsonParser,
        //     input: "1.0e_1",
        //     rule: Rule::float,
        //     positives: [Rule::float],
        //     negatives: [],
        //     pos: 0
        // };
        // fails_with! {
        //     parser: FilsonParser,
        //     input: "1.0e1_",
        //     rule: Rule::float,
        //     positives: [Rule::float],
        //     negatives: [],
        //     pos: 0
        // };
    }

    #[test]
    fn parse_string() {
        parses_to! {
            parser: FilsonParser,
            input: "\"this is a string with $pec1al char's, punctuation and escapes \\ \" \t and maybe crabs 🦀\"",
            rule: Rule::string,
            tokens: [
                string(0, 90, [chars(1, 89)])
            ]
        };
        fails_with! {
            parser: FilsonParser,
            input: "\"this is a an unclosed string",
            rule: Rule::string,
            positives: [Rule::string],
            negatives: [],
            pos: 0
        };
    }

    #[test]
    fn parse_bool() {
        parses_to! {
            parser: FilsonParser,
            input: "true",
            rule: Rule::boolean,
            tokens: [
                boolean(0, 4)
            ]
        };
        parses_to! {
            parser: FilsonParser,
            input: "false",
            rule: Rule::boolean,
            tokens: [
                boolean(0, 5)
            ]
        };
        fails_with! {
            parser: FilsonParser,
            input: "\"true\"",
            rule: Rule::boolean,
            positives: [Rule::boolean],
            negatives: [],
            pos: 0
        };
    }

    #[test]
    fn parse_null() {
        parses_to! {
        parser: FilsonParser,
        input: "null",
        rule: Rule::null,
        tokens: [
            null(0, 4)
            ]
        };
        fails_with! {
            parser: FilsonParser,
            input: "\"null\"",
            rule: Rule::null,
            positives: [Rule::null],
            negatives: [],
            pos: 0
        };
    }
}

#[cfg(test)]
mod test_ops {
    use crate::parser::ast_generation::{FilsonParser, Rule};
    use pest::{consumes_to, parses_to};

    const VALID_OPS: [&str; 6] = ["==", "!=", ">=", "<=", ">", "<"];

    #[test]
    fn test_ops() {
        for op in VALID_OPS {
            parses_to! {
                parser: FilsonParser,
                input: op,
                rule: Rule::operation,
                tokens: [
                    operation(0, op.len())
                ]
            };
        }
    }
}

#[cfg(test)]
mod test_complex_types_parsing {
    use crate::parser::ast_generation::{FilsonParser, Rule};
    use pest::{consumes_to, fails_with, parses_to};

    #[test]
    fn parse_array() {
        parses_to! {
            parser: FilsonParser,
            input: "[]",
            rule: Rule::array,
            tokens: [
                array(0, 2)
            ]
        };
        parses_to! {
            parser: FilsonParser,
            input: "[1, .1e1, \"🦀\", true, null]",
            rule: Rule::array,
            tokens: [
                array(0, 29,
                    [
                        integer(1, 2),
                        float(4, 8),
                        string(10, 16, [chars(11, 15)]),
                        boolean(18, 22),
                        null(24, 28)
                        ]
                    )
            ]
        };
        // Invalid floating point format
        fails_with! {
            parser: FilsonParser,
            input: "[1, .1e_1, \"🦀\", true, null]",
            rule: Rule::array,
            positives: [Rule::string, Rule::null, Rule::boolean],
            negatives: [],
            pos: 4
        };
    }

    #[test]
    fn parse_set() {
        parses_to! {
            parser: FilsonParser,
            input: "{}",
            rule: Rule::set,
            tokens: [
                set(0, 2)
            ]
        };
        parses_to! {
            parser: FilsonParser,
            input: "{1, .1e1, \"🦀\", true, null}",
            rule: Rule::set,
            tokens: [
                set(0, 29,
                    [
                        integer(1, 2),
                        float(4, 8),
                        string(10, 16, [chars(11, 15)]),
                        boolean(18, 22),
                        null(24, 28)
                        ]
                    )
            ]
        };
        // Invalid floating point format
        fails_with! {
            parser: FilsonParser,
            input: "{1, .1e_1, \"🦀\", true, null}",
            rule: Rule::set,
            positives: [Rule::string, Rule::null, Rule::boolean],
            negatives: [],
            pos: 4
        };
    }

    #[test]
    fn parse_map() {
        parses_to! {
            parser: FilsonParser,
            input: "<>",
            rule: Rule::map,
            tokens: [
                map(0, 2)
            ]
        };
        parses_to! {
            parser: FilsonParser,
            input: "< 1: 1, .1e1: .1e1, \"🦀\": \"🦀\", true: true, null: null>",
            rule: Rule::map,
            tokens: [
                map(0, 59,
                    [
                        map_pair(2, 6, [integer(2, 3), integer(5, 6)]),
                        map_pair(8, 18, [float(8, 12), float(14, 18)]),
                        map_pair(20, 34, [string(20, 26, [chars(21, 25)]), string(28, 34, [chars(29, 33)])]),
                        map_pair(36, 46, [boolean(36, 40), boolean(42, 46)]),
                        map_pair(48, 58, [null(48, 52), null(54, 58)]),
                        ]
                    )
            ]
        };
        // Invalid floating point format
        fails_with! {
            parser: FilsonParser,
            input: "< 1: 1, .1e_1: .1e_1, \"🦀\": \"🦀\", true: true, null: null>",
            rule: Rule::map,
            positives: [Rule::map_pair],
            negatives: [],
            pos: 8
        };

        // Collection type can't be a key
        fails_with! {
            parser: FilsonParser,
            input: "< <>: 1, {}: 1, []:1 >",
            rule: Rule::map,
            positives: [Rule::map_pair],
            negatives: [],
            pos: 2
        }
    }
}

#[cfg(test)]
mod test_compare_parsing {
    use crate::parser::ast_generation::{FilsonParser, Rule};
    use pest::{consumes_to, fails_with, parses_to};

    #[test]
    fn test_valid() {
        parses_to! {
           parser: FilsonParser,
            input: r#"compare("/id" == 1)"#,
            rule: Rule::compare,
            tokens: [
                compare(0, 19, [
                    string(8, 13, [chars(9, 12)]),
                    operation(14, 16),
                    integer(17, 18)
                    ]
                )
            ]
        }
    }

    #[test]
    fn test_invalid_right() {
        fails_with! {
           parser: FilsonParser,
            input: r#"compare("/id" == /)"#,
            rule: Rule::compare,
            positives: [Rule::map, Rule::set, Rule::array, Rule::string, Rule::float, Rule::integer, Rule::null, Rule::boolean],
            negatives: [],
            pos: 17
        }
    }

    #[test]
    fn test_invalid_left() {
        fails_with! {
           parser: FilsonParser,
            input: r#"compare(1 == 1)"#,
            rule: Rule::compare,
            positives: [Rule::string],
            negatives: [],
            pos: 8
        }
    }
}

#[cfg(test)]
mod test_intersects_parsing {
    use crate::parser::ast_generation::{FilsonParser, Rule};
    use pest::{consumes_to, fails_with, parses_to};

    #[test]
    fn test_valid() {
        parses_to! {
           parser: FilsonParser,
            input: r#"intersects("/id" [])"#,
            rule: Rule::intersects,
            tokens: [
                intersects(0, 20, [
                    string(11, 16, [chars(12, 15)]),
                    array(17, 19)
                    ]
                )
            ]
        }
    }

    #[test]
    fn test_invalid_left() {
        fails_with! {
           parser: FilsonParser,
            input: r#"intersects(1 [])"#,
            rule: Rule::intersects,
            positives: [Rule::string],
            negatives: [],
            pos: 11
        }
    }

    #[test]
    fn test_invalid_right() {
        fails_with! {
           parser: FilsonParser,
            input: r#"intersects("/id" 1)"#,
            rule: Rule::intersects,
            positives: [Rule::map, Rule::set, Rule::array, Rule::string],
            negatives: [],
            pos: 17
        }
    }
}

#[cfg(test)]
mod test_is_contained_parsing {
    use crate::parser::ast_generation::{FilsonParser, Rule};
    use pest::{consumes_to, fails_with, parses_to};

    #[test]
    fn test_valid() {
        parses_to! {
           parser: FilsonParser,
            input: r#"is_contained("/id" [])"#,
            rule: Rule::is_contained,
            tokens: [
                is_contained(0, 22, [
                    string(13, 18, [chars(14, 17)]),
                    array(19, 21)
                    ]
                )
            ]
        }
    }

    #[test]
    fn test_invalid_left() {
        fails_with! {
           parser: FilsonParser,
            input: r#"is_contained(1 [])"#,
            rule: Rule::is_contained,
            positives: [Rule::string],
            negatives: [],
            pos: 13
        }
    }

    #[test]
    fn test_invalid_right() {
        fails_with! {
           parser: FilsonParser,
            input: r#"is_contained("/id" 1)"#,
            rule: Rule::is_contained,
            positives: [Rule::map, Rule::set, Rule::array],
            negatives: [],
            pos: 19
        }
    }
}

#[cfg(test)]
mod test_exists_parsing {
    use crate::parser::ast_generation::{FilsonParser, Rule};
    use pest::{consumes_to, fails_with, parses_to};

    #[test]
    fn test_valid() {
        parses_to! {
           parser: FilsonParser,
            input: r#"exists("/id")"#,
            rule: Rule::exists,
            tokens: [
                exists(0, 13, [
                    string(7, 12, [chars(8, 11)])
                    ]
                )
            ]
        }
    }

    #[test]
    fn test_invalid() {
        fails_with! {
           parser: FilsonParser,
            input: r#"exists(1)"#,
            rule: Rule::exists,
            positives: [Rule::string],
            negatives: [],
            pos: 7
        }
    }
}

#[cfg(test)]
mod test_is_superset_parsing {
    use crate::parser::ast_generation::{FilsonParser, Rule};
    use pest::{consumes_to, fails_with, parses_to};

    #[test]
    fn test_valid() {
        parses_to! {
           parser: FilsonParser,
            input: r#"is_superset("/id" [])"#,
            rule: Rule::is_superset,
            tokens: [
                is_superset(0, 21, [
                    string(12, 17, [chars(13, 16)]),
                    array(18, 20)
                    ]
                )
            ]
        }
    }

    #[test]
    fn test_invalid_left() {
        fails_with! {
           parser: FilsonParser,
            input: r#"is_superset(1 [])"#,
            rule: Rule::is_superset,
            positives: [Rule::string],
            negatives: [],
            pos: 12
        }
    }

    #[test]
    fn test_invalid_right() {
        fails_with! {
           parser: FilsonParser,
            input: r#"is_superset("/id" 1)"#,
            rule: Rule::is_superset,
            positives: [Rule::map, Rule::set, Rule::array, Rule::string],
            negatives: [],
            pos: 18
        }
    }
}

#[cfg(test)]
mod test_is_subset_parsing {
    use crate::parser::ast_generation::{FilsonParser, Rule};
    use pest::{consumes_to, fails_with, parses_to};

    #[test]
    fn test_valid() {
        parses_to! {
           parser: FilsonParser,
            input: r#"is_subset("/id" [])"#,
            rule: Rule::is_subset,
            tokens: [
                is_subset(0, 19, [
                    string(10, 15, [chars(11, 14)]),
                    array(16, 18)
                    ]
                )
            ]
        }
    }

    #[test]
    fn test_invalid_left() {
        fails_with! {
           parser: FilsonParser,
            input: r#"is_subset(1 [])"#,
            rule: Rule::is_subset,
            positives: [Rule::string],
            negatives: [],
            pos: 10
        }
    }

    #[test]
    fn test_invalid_right() {
        fails_with! {
           parser: FilsonParser,
            input: r#"is_subset("/id" 1)"#,
            rule: Rule::is_subset,
            positives: [Rule::map, Rule::set, Rule::array, Rule::string],
            negatives: [],
            pos: 16
        }
    }
}

#[cfg(test)]
mod test_binary_ops_parsing {
    use crate::parser::ast_generation::{FilsonParser, Rule};
    use pest::{consumes_to, fails_with, parses_to};
    const BIN_OPS: [&str; 3] = ["and", "or", "xor"];

    #[test]
    fn test_valid_identifiers() {
        for ident in BIN_OPS {
            parses_to! {
                parser: FilsonParser,
                input: ident,
                rule: Rule::binary_identifier,
                tokens: [binary_identifier(0, ident.len())]
            }
        }
    }

    #[test]
    fn test_body_valid_no_nesting() {
        parses_to! {
        parser: FilsonParser,
        input:r#"(exists("/id"), exists("/id"))"#,
        rule: Rule::binary_body,
        tokens: [
            binary_body(0, 30, [
                exists(1, 14, [string(8, 13, [chars(9, 12)])]),
                exists(16, 29, [string(23, 28, [chars(24, 27)])])
        ])
        ]}
    }

    #[test]
    fn test_body_valid_with_nesting() {
        parses_to! {
        parser: FilsonParser,
        input:r#"(and(exists("/id"), exists("/id")), exists("/id"))"#,
        rule: Rule::binary_body,
        tokens: [
            binary_body(0, 50, [
                binary_operation(1, 34, [
                    binary_identifier(1, 4),
                    binary_body(4, 34, [
                        exists(5, 18, [string(12, 17, [chars(13, 16)])]),
                        exists(20, 33, [string(27, 32, [chars(28, 31)])]),
                    ])
                ]),
                exists(36, 49, [string(43, 48, [chars(44, 47)])]),
        ])
        ]}
    }

    #[test]
    fn test_body_invalid_no_nesting() {
        fails_with! {
            parser: FilsonParser,
            input:r#"(karl, exists("/id"))"#,
            rule: Rule::binary_body,
            positives: [Rule::not, Rule::binary_identifier, Rule::compare,
                        Rule::intersects, Rule::is_contained,
                        Rule::exists, Rule::is_superset, Rule::is_subset],
            negatives: [],
            pos: 1
        }
    }
}

#[cfg(test)]
mod test_ast_generation {
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
