use crate::objects::shared::{Object, ObjectType};

pub struct Blob {
    pub data: Vec<u8>,
}

impl Object for Blob {
    fn object_type(&self) -> ObjectType {
        ObjectType::Blob
    }

    fn deserialize(data: &[u8]) -> Self {
        Blob {
            data: data.to_vec(),
        }
    }

    fn serialize(&self) -> Vec<u8> {
        self.data.clone()
    }
}
