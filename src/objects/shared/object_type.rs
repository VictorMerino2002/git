use std::fmt::Display;

use anyhow::{Result, bail};

pub enum ObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            ObjectType::Commit => "commit",
            ObjectType::Tree => "tree",
            ObjectType::Blob => "blob",
            ObjectType::Tag => "tag",
        };
        write!(f, "{string}")
    }
}

impl TryFrom<String> for ObjectType {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "commit" => Ok(ObjectType::Commit),
            "tree" => Ok(ObjectType::Tree),
            "blob" => Ok(ObjectType::Blob),
            "tag" => Ok(ObjectType::Tag),
            _ => bail!("Unknown object type: {}", value),
        }
    }
}
