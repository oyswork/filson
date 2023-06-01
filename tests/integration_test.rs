mod setup;

mod common {
    use super::*;
    use filson::{get_filter, Appliable, FilsonError};

    pub(super) fn run_singlet_test(singlet: &str, expected: Vec<Result<bool, FilsonError>>) {
        let flt = get_filter(singlet).unwrap();
        let test_data = setup::get_test_data();
        let actual = test_data.iter().map(|d| flt.apply(d)).collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    pub(super) fn run_doublet_test<F: Fn((&str, &str)) -> String>(
        doublet: (&str, &str),
        expected: Vec<Result<bool, FilsonError>>,
        construction_func: F,
    ) {
        let cond = construction_func(doublet);
        let flt = get_filter(&cond).unwrap();
        let test_data = setup::get_test_data();
        let actual = test_data.iter().map(|d| flt.apply(d)).collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    pub(super) fn run_triplet_test<F: Fn((&str, &str, &str)) -> String>(
        triplet: (&str, &str, &str),
        expected: Vec<Result<bool, FilsonError>>,
        construction_func: F,
    ) {
        let cond = construction_func(triplet);
        let flt = get_filter(&cond).unwrap();
        let test_data = setup::get_test_data();
        let actual = test_data.iter().map(|d| flt.apply(d)).collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod test_compare {
    use super::*;
    use filson::FilsonError;

    fn construct_compare_eq_from_doublet(doublet: (&str, &str)) -> String {
        format!(r#"compare("{}" == {})"#, doublet.0, doublet.1)
    }

    #[test]
    fn compare_with_all_possible_rhs_types_matching_lhs() {
        let doublets = [
            ("int", "1"),
            ("float", "1.0"),
            ("text", r#""test text""#),
            ("boolean", "true"),
            ("map", r#"<"first": 1, "second": 2>"#),
            ("set", "{1, 2}"),
            ("array", "[1, 2]"),
        ];
        for doublet in doublets {
            common::run_doublet_test(
                doublet,
                vec![Ok(true), Ok(false)],
                construct_compare_eq_from_doublet,
            );
        }
        common::run_doublet_test(
            ("null", "null"),
            vec![Ok(true), Ok(true)],
            construct_compare_eq_from_doublet,
        );
    }

    #[test]
    fn compare_with_all_possible_rhs_types_wrong_lhs() {
        let doublets = [
            ("int", "1.0"),
            ("float", r#""test text""#),
            ("text", "1"),
            ("boolean", "null"),
            ("map", "{1, 2}"),
            ("set", "[1, 2]"),
            ("array", r#"<"first": 1, "second": 2>"#),
            ("null", "true"),
        ];
        for doublet in doublets {
            common::run_doublet_test(
                doublet,
                vec![Err(FilsonError::TypeError), Err(FilsonError::TypeError)],
                construct_compare_eq_from_doublet,
            );
        }
    }
}

#[cfg(test)]
mod test_intersects {
    use super::*;
    use filson::{get_filter, FilsonError};

    fn construct_intersects_from_doublet(doublet: (&str, &str)) -> String {
        format!(r#"intersects("{}" {})"#, doublet.0, doublet.1)
    }

    #[test]
    fn intersects_with_correct_types() {
        let doublets = [
            ("map", r#"<"third": 3>"#),
            ("set", "{3}"),
            ("array", "[3]"),
            ("text", r#""karl""#),
        ];
        for doublet in doublets {
            common::run_doublet_test(
                doublet,
                vec![Ok(false), Ok(true)],
                construct_intersects_from_doublet,
            );
        }
    }

    #[test]
    fn intersects_with_unmatching_types() {
        let doublets = [
            ("set", r#"<"third": 3>"#),
            ("map", "{3}"),
            ("text", "[3]"),
            ("array", r#""2""#),
        ];
        for doublet in doublets {
            common::run_doublet_test(
                doublet,
                vec![Err(FilsonError::TypeError), Err(FilsonError::TypeError)],
                construct_intersects_from_doublet,
            );
        }
    }

    #[test]
    fn intersects_with_illegal_lhs_types() {
        let doublets = [
            ("int", r#"<"third": 3>"#),
            ("float", "{3}"),
            ("boolean", "[3]"),
            ("null", r#""2""#),
        ];
        for doublet in doublets {
            common::run_doublet_test(
                doublet,
                vec![
                    Err(FilsonError::IntersectsError),
                    Err(FilsonError::IntersectsError),
                ],
                construct_intersects_from_doublet,
            );
        }
    }

    #[test]
    fn intersects_with_illegal_rhs_types() {
        // shouldn't be able to parse
        // parser should not allow for incorrect types
        let doublets = [
            ("int", "1"),
            ("float", "1.0"),
            ("boolean", "true"),
            ("null", "null"),
        ];
        for doublet in doublets {
            assert!(get_filter(&construct_intersects_from_doublet(doublet)).is_err());
        }
    }
}

#[cfg(test)]
mod test_is_contained {
    use super::*;
    use filson::get_filter;

    fn construct_is_contained_from_doublet(doublet: (&str, &str)) -> String {
        format!(r#"is_contained("{}" {})"#, doublet.0, doublet.1)
    }

    #[test]
    fn is_contained_correct_rhs() {
        let lhsses = ["int", "float", "text", "boolean"];
        let rhsses = [
            r#"<1: null, 1.0: null, "test text": null, true: null>"#,
            r#"{1, 1.0, "test text" , true, <"first": 1, "second": 2>, {1, 2}, [1, 2]}"#,
            r#"[1, 1.0, "test text" , true, <"first": 1, "second": 2>, {1, 2}, [1, 2]]"#,
        ];
        for lhs in lhsses {
            for rhs in rhsses {
                common::run_doublet_test(
                    (lhs, rhs),
                    vec![Ok(true), Ok(false)],
                    construct_is_contained_from_doublet,
                );
            }
        }
        let null_rhses = ["<null: null>", "{null}", "[null]"];
        for null_rhs in null_rhses {
            common::run_doublet_test(
                ("null", null_rhs),
                vec![Ok(true), Ok(true)],
                construct_is_contained_from_doublet,
            );
        }
    }

    #[test]
    fn is_contained_illegal_rhs() {
        // shouldn't be able to parse
        // parser should not allow for incorrect types
        let rhsses = ["1", "1.0", r#""some text""#, "true", "null"];
        for rhs in rhsses {
            assert!(get_filter(&construct_is_contained_from_doublet(("whatever", rhs))).is_err());
        }
    }
}

#[cfg(test)]
mod test_exists {
    use filson::{get_filter, Appliable};

    use super::*;

    fn construct_exists(path: &str) -> String {
        format!(r#"exists("{}")"#, path)
    }

    #[test]
    fn actually_exists() {
        let existing_paths = [
            "int", "float", "text", "boolean", "map", "set", "array", "null",
        ];
        let test_data = setup::get_test_data();
        for path in existing_paths {
            let cond = construct_exists(path);
            let flt = get_filter(&cond).unwrap();
            let actual = test_data.iter().map(|d| flt.apply(d)).collect::<Vec<_>>();
            assert_eq!(actual, vec![Ok(true), Ok(true)]);
        }
    }

    #[test]
    fn doesnt_exist() {
        let fake_path = "fake";
        let test_data = setup::get_test_data();
        let cond = construct_exists(fake_path);
        let flt = get_filter(&cond).unwrap();
        let actual = test_data.iter().map(|d| flt.apply(d)).collect::<Vec<_>>();
        assert_eq!(actual, vec![Ok(false), Ok(false)]);
    }
}

#[cfg(test)]
mod test_is_subset {
    use super::*;
    use filson::{get_filter, FilsonError};

    fn construct_is_subset_from_doublet(doublet: (&str, &str)) -> String {
        format!(r#"is_subset("{}" {})"#, doublet.0, doublet.1)
    }

    #[test]
    fn is_subset_legal_lhs_rhs() {
        let doublets = [
            ("array", "[2, 3, 5]"),
            ("set", "{2, 3, 5}"),
            ("map", r#"<"second": 2, "third": 3, "fifth": 5>"#),
            ("text", r#""karlissimo""#),
        ];
        for doublet in doublets {
            common::run_doublet_test(
                doublet,
                vec![Ok(false), Ok(true)],
                construct_is_subset_from_doublet,
            );
        }
    }

    #[test]
    fn is_subset_illegal_lhs() {
        let doublets = [
            ("int", "[1, 2, 5]"),
            ("float", "{1, 2, 5}"),
            ("boolean", r#"<"first": 1, "second": 2, "fifth": 5>"#),
        ];
        for doublet in doublets {
            common::run_doublet_test(
                doublet,
                vec![
                    Err(FilsonError::IsSubsetError),
                    Err(FilsonError::IsSubsetError),
                ],
                construct_is_subset_from_doublet,
            );
        }
    }

    #[test]
    fn is_subset_illegal_rhs() {
        // shouldn't be able to parse
        // parser should not allow for incorrect types
        let rhsses = ["1", "1.0", "true", "null"];
        for rhs in rhsses {
            assert!(get_filter(&construct_is_subset_from_doublet(("whatever", rhs))).is_err());
        }
    }

    #[test]
    fn is_subset_unmatching_lhs_rhs() {
        let doublets = [
            ("set", "[2, 3, 5]"),
            ("array", "{2, 3, 5}"),
            ("text", r#"<"second": 2, "third": 3, "fifth": 5>"#),
            ("map", r#""karlissimo""#),
        ];
        for doublet in doublets {
            common::run_doublet_test(
                doublet,
                vec![Err(FilsonError::TypeError), Err(FilsonError::TypeError)],
                construct_is_subset_from_doublet,
            );
        }
    }
}

#[cfg(test)]
mod test_is_superset {
    use super::*;
    use filson::{get_filter, FilsonError};

    fn construct_is_superset_from_doublet(doublet: (&str, &str)) -> String {
        format!(r#"is_superset("{}" {})"#, doublet.0, doublet.1)
    }

    #[test]
    fn is_superset_legal_lhs_rhs() {
        let doublets = [
            ("array", "[3]"),
            ("set", "{3}"),
            ("map", r#"<"third": 3>"#),
            ("text", r#""ka""#),
        ];
        for doublet in doublets {
            common::run_doublet_test(
                doublet,
                vec![Ok(false), Ok(true)],
                construct_is_superset_from_doublet,
            );
        }
    }

    #[test]
    fn is_superset_illegal_lhs() {
        let doublets = [
            ("int", "[1, 2, 5]"),
            ("float", "{1, 2, 5}"),
            ("boolean", r#"<"first": 1, "second": 2, "fifth": 5>"#),
        ];
        for doublet in doublets {
            common::run_doublet_test(
                doublet,
                vec![
                    Err(FilsonError::IsSupersetError),
                    Err(FilsonError::IsSupersetError),
                ],
                construct_is_superset_from_doublet,
            );
        }
    }

    #[test]
    fn is_superset_illegal_rhs() {
        // shouldn't be able to parse
        // parser should not allow for incorrect types
        let rhsses = ["1", "1.0", "true", "null"];
        for rhs in rhsses {
            assert!(get_filter(&construct_is_superset_from_doublet(("whatever", rhs))).is_err());
        }
    }

    #[test]
    fn is_superset_unmatching_lhs_rhs() {
        let doublets = [
            ("set", "[2, 3, 5]"),
            ("array", "{2, 3, 5}"),
            ("text", r#"<"second": 2, "third": 3, "fifth": 5>"#),
            ("map", r#""karlissimo""#),
        ];
        for doublet in doublets {
            common::run_doublet_test(
                doublet,
                vec![Err(FilsonError::TypeError), Err(FilsonError::TypeError)],
                construct_is_superset_from_doublet,
            );
        }
    }
}

#[cfg(test)]
mod test_binary_conditions {
    use filson::get_filter;

    use super::*;

    fn construct_binary_from_triplet(triplet: (&str, &str, &str)) -> String {
        format!("{}({}, {})", triplet.0, triplet.1, triplet.2)
    }

    #[test]
    fn and_legal_rhs_lhs() {
        common::run_triplet_test(
            (
                "and",
                r#"is_superset("array" [2])"#,
                r#"compare("int" == 1)"#,
            ),
            vec![Ok(true), Ok(false)],
            construct_binary_from_triplet,
        )
    }

    #[test]
    fn or_legal_rhs_lhs() {
        common::run_triplet_test(
            (
                "or",
                r#"is_superset("array" [2])"#,
                r#"compare("int" == 1)"#,
            ),
            vec![Ok(true), Ok(true)],
            construct_binary_from_triplet,
        )
    }

    #[test]
    fn xor_legal_rhs_lhs() {
        common::run_triplet_test(
            (
                "xor",
                r#"is_superset("array" [2])"#,
                r#"compare("int" == 1)"#,
            ),
            vec![Ok(false), Ok(true)],
            construct_binary_from_triplet,
        )
    }

    #[test]
    fn binary_illegal_rhs() {
        // shouldn't be able to parse
        // parser should not allow for incorrect types
        let triplets = [
            ("and", "1", r#"compare("int" == 1)"#),
            ("or", "1", r#"compare("int" == 1)"#),
            ("xor", "1", r#"compare("int" == 1)"#),
            ("and", r#"compare("int" == 1)"#, "1"),
            ("or", r#"compare("int" == 1)"#, "1"),
            ("xor", r#"compare("int" == 1)"#, "1"),
        ];
        for triplet in triplets {
            assert!(get_filter(&construct_binary_from_triplet(triplet)).is_err());
        }
    }

    #[test]
    fn nesting_legal_rhs_lhs() {
        common::run_triplet_test(
            (
                "xor",
                r#"or(is_superset("array" [3]), compare("int" == 1))"#,
                r#"and(compare("int" == 2), is_subset("text" "karlissimo"))"#,
            ),
            vec![Ok(true), Ok(false)],
            construct_binary_from_triplet,
        )
    }
}

#[cfg(test)]
mod test_negation {
    use super::*;
    use filson::get_filter;

    #[test]
    fn negates_valid_thing() {
        let conditions = [
            r#"!compare("int" == 1)"#,
            r#"!intersects("text" "text")"#,
            r#"!is_contained("int" [1, 3])"#,
            r#"!is_subset("array" [1, 2, 5])"#,
            r#"!is_superset("array" [1])"#,
            r#"!and(is_contained("int" [1, 2, 3]), compare("int" == 1))"#,
            r#"!or(compare("float" == 1.0), compare("int" == 1))"#,
            r#"!xor(is_contained("int" [1, 3]), is_contained("int" [3, 5]))"#,
        ];
        for cond in conditions {
            common::run_singlet_test(cond, vec![Ok(false), Ok(true)]);
        }
        common::run_singlet_test(r#"!exists("int")"#, vec![Ok(false), Ok(false)]);
        common::run_singlet_test(r#"!exists("fake")"#, vec![Ok(true), Ok(true)]);
    }

    #[test]
    fn negates_invalid_thing() {
        // shouldn't be able to parse
        // parser should not allow for incorrect types
        assert!(get_filter("!1").is_err());
    }
}
