use std::fmt::Display;

pub enum ObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
}

pub trait Object {
    fn object_type(&self) -> ObjectType;
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self
    where
        Self: Sized;
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
