use std::collections::{BTreeMap, BTreeSet};

use filson::{DataNode, Extractable};

pub(crate) struct TestStruct {
    int: i64,
    float: f64,
    text: &'static str,
    boolean: bool,
    map: BTreeMap<&'static str, i64>,
    set: BTreeSet<i64>,
    array: Vec<i64>,
}

impl Extractable for TestStruct {
    fn extract(&self, path: &str) -> Result<filson::DataNode, filson::FilsonError> {
        match path {
            "int" => Ok(self.int.into()),
            "float" => Ok(self.float.into()),
            "text" => Ok(self.text.into()),
            "boolean" => Ok(self.boolean.into()),
            "null" => Ok(DataNode::Null),
            "map" => Ok(self
                .map
                .iter()
                .map(|(k, v)| ((*k).into(), (*v).into()))
                .collect::<BTreeMap<_, _>>()
                .into()),
            "set" => Ok(self
                .set
                .iter()
                .map(|x| (*x).into())
                .collect::<BTreeSet<_>>()
                .into()),
            "array" => Ok(self
                .array
                .iter()
                .map(|x| (*x).into())
                .collect::<Vec<_>>()
                .into()),
            _ => Err(filson::FilsonError::ExtractionError),
        }
    }
}

pub(crate) fn get_test_data() -> Vec<TestStruct> {
    let one = TestStruct {
        int: 1,
        float: 1.0,
        text: "test text",
        boolean: true,
        map: BTreeMap::from_iter(vec![("first", 1), ("second", 2)]),
        set: BTreeSet::from_iter(vec![1, 2]),
        array: vec![1, 2],
    };
    let two = TestStruct {
        int: 2,
        float: 2.0,
        text: "karl",
        boolean: false,
        map: BTreeMap::from_iter(vec![("second", 2), ("third", 3)]),
        set: BTreeSet::from_iter(vec![2, 3]),
        array: vec![2, 3],
    };
    vec![one, two]
}
