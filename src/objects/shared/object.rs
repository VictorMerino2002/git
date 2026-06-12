use crate::objects::shared::ObjectType;
use std::any::Any;

pub trait Object {
    fn object_type(&self) -> ObjectType;
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self
    where
        Self: Sized;
    fn as_any(&self) -> &dyn Any;
}
