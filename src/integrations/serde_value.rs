use std::collections::BTreeMap;

use crate::{error::FilsonResult, DataNode, Extractable, FilsonError};

impl<'a> From<&'a serde_json::Value> for DataNode<'a> {
    fn from(value: &'a serde_json::Value) -> Self {
        match value {
            serde_json::Value::Bool(b) => (*b).into(),
            serde_json::Value::Number(x) => {
                if x.is_i64() {
                    x.as_i64().unwrap().into()
                } else {
                    x.as_f64().unwrap().into()
                }
            }
            serde_json::Value::String(x) => x.as_str().into(),
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Array(arr) => arr.iter().map(Self::from).collect::<Vec<_>>().into(),
            serde_json::Value::Object(obj) => obj
                .iter()
                .map(|(key, value)| (Self::from(key.as_str()), Self::from(value)))
                .collect::<BTreeMap<_, _>>()
                .into(),
        }
    }
}

impl Extractable for serde_json::Value {
    fn extract(&self, path: &str) -> FilsonResult<DataNode> {
        let v = self.pointer(path).ok_or(FilsonError::ExtractionError)?;
        Ok(DataNode::from(v))
    }
}
