use std::collections::{BTreeMap, HashSet};
use std::hash::Hash;

use crate::DataNode;

/// Checks for intersection corner cases between two collections.
///
/// It is considered a valid intersection or a subset when left is empty and right isn't.
/// It also checks the case when right is empty and left isn't, which can't be a valid intersection/subset.
///
/// returns ```Option(bool)``` when one of the above cases is met and ```None``` when neither.
#[inline]
fn check_intersection_or_subset_corner_cases<T: ExactSizeIterator>(
    left: &T,
    right: &T,
) -> Option<bool> {
    if left.len() == 0 {
        return Some(true);
    } else if (left.len() != 0) & (right.len() == 0) {
        return Some(false);
    }
    None
}

fn btreemap_intersects(
    left: &BTreeMap<DataNode, DataNode>,
    right: &BTreeMap<DataNode, DataNode>,
) -> bool {
    if let Some(corner_case) =
        check_intersection_or_subset_corner_cases(&left.iter(), &right.iter())
    {
        return corner_case;
    };
    for (key_left, val_left) in left {
        if let Some(val_right) = right.get(key_left) {
            if val_left == val_right {
                return true;
            }
        }
    }
    false
}

fn str_array_intersects<Y, T>(mut left: T, right: T) -> bool
where
    Y: Eq + Hash,
    T: ExactSizeIterator<Item = Y>,
{
    if let Some(corner_case) = check_intersection_or_subset_corner_cases(&left, &right) {
        return corner_case;
    };
    let right_set: HashSet<Y> = HashSet::from_iter(right);
    left.any(|element| right_set.contains(&element))
}

fn array_is_subset(left: &[DataNode], right: &[DataNode]) -> bool {
    if let Some(corner_case) =
        check_intersection_or_subset_corner_cases(&left.iter(), &right.iter())
    {
        return corner_case;
    };
    let left_len = left.len();
    if left.len() > right.len() {
        return false;
    }
    for window in right.windows(left_len) {
        if window == left {
            return true;
        }
    }
    false
}

fn map_is_subset(
    left: &BTreeMap<DataNode, DataNode>,
    right: &BTreeMap<DataNode, DataNode>,
) -> bool {
    if let Some(corner_case) =
        check_intersection_or_subset_corner_cases(&left.iter(), &right.iter())
    {
        return corner_case;
    };
    if left.len() > right.len() {
        return false;
    }
    for (key_left, val_left) in left {
        if let Some(val_right) = right.get(key_left) {
            if val_left != val_right {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

// parser won't allow for incorrect types
// this will be called from top level actor, where it is checked if descriminants are the same
pub(super) fn intersects_logic(left: &DataNode, right: &DataNode) -> bool {
    let res = match (left, right) {
        (DataNode::Set(left_set), DataNode::Set(right_set)) => {
            left_set.intersection(right_set).next().is_some()
        }
        (DataNode::Array(left_arr), DataNode::Array(right_arr)) => {
            str_array_intersects(left_arr.iter(), right_arr.iter())
        }
        (DataNode::Map(left_map), DataNode::Map(right_map)) => {
            btreemap_intersects(left_map, right_map)
        }
        (DataNode::Str(left_str), DataNode::Str(right_str)) => {
            str_array_intersects(left_str.bytes(), right_str.bytes())
        }
        _ => unreachable!(),
    };
    res
}

// parser won't allow for incorrect types
// this will be called from top level actor, where it is checked if descriminants are the same
pub(super) fn is_subset_logic(left: &DataNode, right: &DataNode) -> bool {
    match (left, right) {
        (DataNode::Set(left_set), DataNode::Set(right_set)) => left_set.is_subset(right_set),
        (DataNode::Array(left_arr), DataNode::Array(right_arr)) => {
            array_is_subset(left_arr, right_arr)
        }
        (DataNode::Map(left_map), DataNode::Map(right_map)) => map_is_subset(left_map, right_map),
        (DataNode::Str(left_str), DataNode::Str(right_str)) => right_str.contains(left_str),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests_map_intersects {
    use super::*;
    #[test]
    fn test_map_intersects_maps_intersect() {
        let mut left_map = BTreeMap::new();
        let mut right_map = BTreeMap::new();

        left_map.insert(DataNode::from(1), DataNode::from(10));
        left_map.insert(DataNode::from(2), DataNode::from(20));

        right_map.insert(DataNode::from(2), DataNode::from(20));
        right_map.insert(DataNode::from(4), DataNode::from(40));

        assert!(btreemap_intersects(&left_map, &right_map));
    }

    #[test]
    fn test_map_intersects_maps_do_not_intersect() {
        let mut left_map = BTreeMap::new();
        let mut right_map = BTreeMap::new();

        left_map.insert(DataNode::from(1), DataNode::from(10));
        left_map.insert(DataNode::from(2), DataNode::from(20));

        right_map.insert(DataNode::from(3), DataNode::from(30));
        right_map.insert(DataNode::from(4), DataNode::from(40));

        assert!(!btreemap_intersects(&left_map, &right_map));
    }

    #[test]
    fn test_map_intersects_have_same_key_but_different_values() {
        let mut left_map = BTreeMap::new();
        let mut right_map = BTreeMap::new();

        left_map.insert(DataNode::from(1), DataNode::from(10));
        left_map.insert(DataNode::from(2), DataNode::from(20));

        right_map.insert(DataNode::from(2), DataNode::from(200));
        right_map.insert(DataNode::from(3), DataNode::from(30));

        assert!(!btreemap_intersects(&left_map, &right_map));
    }

    #[test]
    fn test_map_intersects_have_same_value_but_different_keys() {
        let mut left_map = BTreeMap::new();
        let mut right_map = BTreeMap::new();

        left_map.insert(DataNode::from(1), DataNode::from(10));
        left_map.insert(DataNode::from(2), DataNode::from(20));

        right_map.insert(DataNode::from(20), DataNode::from(20));
        right_map.insert(DataNode::from(3), DataNode::from(30));

        assert!(!btreemap_intersects(&left_map, &right_map));
    }

    #[test]
    fn test_map_intersects_left_map_is_empty() {
        let left_map = BTreeMap::new();
        let mut right_map = BTreeMap::new();

        right_map.insert(DataNode::from(2), DataNode::from(20));
        right_map.insert(DataNode::from(4), DataNode::from(40));

        assert!(btreemap_intersects(&left_map, &right_map));
    }

    #[test]
    fn test_map_intersects_right_map_is_empty() {
        let mut left_map = BTreeMap::new();
        let right_map = BTreeMap::new();

        left_map.insert(DataNode::from(1), DataNode::from(10));
        left_map.insert(DataNode::from(2), DataNode::from(20));
        left_map.insert(DataNode::from(3), DataNode::from(30));

        assert!(!btreemap_intersects(&left_map, &right_map));
    }

    #[test]
    fn test_map_intersects_both_maps_are_empty() {
        let left_map = BTreeMap::new();
        let right_map = BTreeMap::new();

        assert!(btreemap_intersects(&left_map, &right_map));
    }
}

#[cfg(test)]
mod tests_array_intersects {
    use super::*;
    #[test]
    fn test_array_intersects() {
        let left_arr = vec![DataNode::from(1), DataNode::from(2)];
        let right_arr = vec![DataNode::from(2), DataNode::from(4)];

        assert!(str_array_intersects(left_arr.iter(), right_arr.iter()));
    }

    #[test]
    fn test_arrays_do_not_intersect() {
        let left_arr = vec![DataNode::from(1), DataNode::from(2)];
        let right_arr = vec![DataNode::from(4), DataNode::from(5)];

        assert!(!str_array_intersects(left_arr.iter(), right_arr.iter()));
    }

    #[test]
    fn test_array_intersects_left_is_empty() {
        let left_arr = vec![];
        let right_arr = vec![DataNode::from(2), DataNode::from(4)];

        assert!(str_array_intersects(left_arr.iter(), right_arr.iter()));
    }

    #[test]
    fn test_array_intersects_right_is_empty() {
        let left_arr = vec![DataNode::from(1), DataNode::from(2), DataNode::from(3)];
        let right_arr = vec![];

        assert!(!str_array_intersects(left_arr.iter(), right_arr.iter()));
    }

    #[test]
    fn test_array_intersects_both_are_empty() {
        let left_arr: Vec<u8> = vec![];
        let right_arr: Vec<u8> = vec![];

        assert!(str_array_intersects(left_arr.iter(), right_arr.iter()));
    }
}

#[cfg(test)]
mod tests_str_intersects {
    use super::*;
    #[test]
    fn test_str_intersects_both_empty() {
        let left_str = "";
        let right_str = "";
        assert!(str_array_intersects(left_str.bytes(), right_str.bytes()));
    }

    #[test]
    fn test_str_intersects_left_empty() {
        let left_str = "";
        let right_str = "hello";
        assert!(str_array_intersects(left_str.bytes(), right_str.bytes()));
    }

    #[test]
    fn test_str_intersects_right_empty() {
        let left_str = "world";
        let right_str = "";
        assert!(!str_array_intersects(left_str.bytes(), right_str.bytes()));
    }

    #[test]
    fn test_str_no_intersection() {
        let left_str = "abc";
        let right_str = "def";
        assert!(!str_array_intersects(left_str.bytes(), right_str.bytes()));
    }

    #[test]
    fn test_str_intersection() {
        let left_str = "abc";
        let right_str = "bcd";
        assert!(str_array_intersects(left_str.bytes(), right_str.bytes()));
    }
}

#[cfg(test)]
mod tests_array_subset {
    use super::*;
    #[test]
    fn test_array_is_subset() {
        let left_arr = vec![DataNode::from(1), DataNode::from(2)];
        let right_arr = vec![
            DataNode::from(1),
            DataNode::from(2),
            DataNode::from(3),
            DataNode::from(4),
        ];

        assert!(array_is_subset(&left_arr, &right_arr));
    }

    #[test]
    fn test_array_is_subset_returns_false_when_left_array_is_not_subset_of_right_array() {
        let left_arr = vec![DataNode::from(1), DataNode::from(2)];
        let right_arr = vec![
            DataNode::from(2),
            DataNode::from(3),
            DataNode::from(4),
            DataNode::from(5),
        ];

        assert!(!array_is_subset(&left_arr, &right_arr));
    }

    #[test]
    fn test_array_is_subset_left_empty() {
        let left_arr = vec![];
        let right_arr = vec![DataNode::from(4), DataNode::from(5)];

        assert!(array_is_subset(&left_arr, &right_arr));
    }

    #[test]
    fn test_array_is_subset_right_array_empty() {
        let left_arr = vec![DataNode::from(1), DataNode::from(2)];
        let right_arr = vec![];

        assert!(!array_is_subset(&left_arr, &right_arr));
    }

    #[test]
    fn test_array_is_subset_returns_true_when_both_arrays_are_empty() {
        let left_arr = vec![];
        let right_arr = vec![];

        assert!(array_is_subset(&left_arr, &right_arr));
    }

    #[test]
    fn test_array_is_subset_left_is_identical_to_right() {
        let left_arr = vec![DataNode::from(1), DataNode::from(2)];
        let right_arr = left_arr.clone();
        assert!(array_is_subset(&left_arr, &right_arr));
    }
}

#[cfg(test)]
mod test_map_subset {
    use super::*;

    #[test]
    fn test_map_is_subset_left_map_is_subset_of_right_map() {
        let mut left_map = BTreeMap::new();
        left_map.insert(DataNode::from(1), DataNode::from(10));
        left_map.insert(DataNode::from(2), DataNode::from(20));

        let mut right_map = BTreeMap::new();
        right_map.insert(DataNode::from(1), DataNode::from(10));
        right_map.insert(DataNode::from(2), DataNode::from(20));
        right_map.insert(DataNode::from(3), DataNode::from(30));

        assert!(map_is_subset(&left_map, &right_map));
    }

    #[test]
    fn test_map_is_subset_left_map_is_not_subset_of_right_map() {
        let mut left_map = BTreeMap::new();
        left_map.insert(DataNode::from(1), DataNode::from(10));
        left_map.insert(DataNode::from(2), DataNode::from(20));
        left_map.insert(DataNode::from(3), DataNode::from(30));

        let mut right_map = BTreeMap::new();
        right_map.insert(DataNode::from(1), DataNode::from(10));
        right_map.insert(DataNode::from(2), DataNode::from(20));

        assert!(!map_is_subset(&left_map, &right_map));
    }

    #[test]
    fn test_map_is_subset_left_map_is_empty() {
        let left_map = BTreeMap::new();

        let mut right_map = BTreeMap::new();
        right_map.insert(DataNode::from(2), DataNode::from(20));
        right_map.insert(DataNode::from(1), DataNode::from(10));

        assert!(map_is_subset(&left_map, &right_map));
    }

    #[test]
    fn test_map_is_subset_right_map_is_empty() {
        let mut left_map = BTreeMap::new();
        left_map.insert(DataNode::from(1), DataNode::from(10));
        left_map.insert(DataNode::from(2), DataNode::from(20));

        let right_map = BTreeMap::new();

        assert!(!map_is_subset(&left_map, &right_map));
    }

    #[test]
    fn test_map_is_subset_both_maps_are_empty() {
        let left_map = BTreeMap::new();
        let right_map = BTreeMap::new();

        assert!(map_is_subset(&left_map, &right_map));
    }

    #[test]
    fn test_map_is_subset_same_keys_different_values() {
        let mut left_map = BTreeMap::new();
        left_map.insert(DataNode::from(1), DataNode::from(10));
        left_map.insert(DataNode::from(2), DataNode::from(20));

        let mut right_map = BTreeMap::new();
        right_map.insert(DataNode::from(1), DataNode::from(100));
        right_map.insert(DataNode::from(2), DataNode::from(200));
        right_map.insert(DataNode::from(3), DataNode::from(300));

        assert!(!map_is_subset(&left_map, &right_map));
    }

    #[test]
    fn test_map_is_subset_same_values_different_keys() {
        let mut left_map = BTreeMap::new();
        left_map.insert(DataNode::from(1), DataNode::from(10));
        left_map.insert(DataNode::from(2), DataNode::from(20));

        let mut right_map = BTreeMap::new();
        right_map.insert(DataNode::from(10), DataNode::from(10));
        right_map.insert(DataNode::from(20), DataNode::from(20));
        right_map.insert(DataNode::from(3), DataNode::from(300));

        assert!(!map_is_subset(&left_map, &right_map));
    }
}
