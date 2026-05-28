use crate::objects::shared::ObjectType;

pub trait Object {
    fn object_type(&self) -> ObjectType;
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self
    where
        Self: Sized;
}
