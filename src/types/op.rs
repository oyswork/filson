#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Op {
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
}

impl<T: AsRef<str>> From<T> for Op {
    fn from(value: T) -> Self {
        match value.as_ref() {
            "==" => Self::Eq,
            "!=" => Self::Ne,
            ">=" => Self::Gte,
            "<=" => Self::Lte,
            ">" => Self::Gt,
            "<" => Self::Lt,
            _ => unreachable!(),
        }
    }
}

impl Op {
    #[cfg(not(feature = "collection_ordering"))]
    pub fn is_ordering(&self) -> bool {
        if let Op::Eq | Op::Ne = self {
            false
        } else {
            true
        }
    }
}
