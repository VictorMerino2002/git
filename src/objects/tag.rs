use std::any::Any;

use crate::{
    objects::shared::{Object, ObjectType},
    utils::kvlm::{Kvlm, kvlm_parse, kvlm_serialize},
};

pub struct Tag {
    pub object: String,
    pub object_type: ObjectType,
    pub tag: String,
    pub tagger: String,
    pub message: String,
}

impl Object for Tag {
    fn object_type(&self) -> ObjectType {
        ObjectType::Tag
    }

    fn serialize(&self) -> Vec<u8> {
        let mut map = Kvlm::new();
        map.insert("object".into(), vec![self.object.clone()]);
        map.insert("type".into(), vec![self.object_type.to_string()]);
        map.insert("tag".into(), vec![self.tag.clone()]);
        map.insert("message".into(), vec![self.message.clone()]);

        kvlm_serialize(&map).into_bytes()
    }

    fn deserialize(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        let raw = str::from_utf8(data).expect("Invalid UTF-8");
        let map = kvlm_parse(raw);

        Self {
            object: map.get("object").and_then(|v| v.first()).cloned().unwrap(),
            object_type: ObjectType::try_from(
                map.get("type").and_then(|v| v.first()).cloned().unwrap(),
            )
            .unwrap(),
            tag: map.get("tag").and_then(|v| v.first()).cloned().unwrap(),
            tagger: map.get("tagger").and_then(|v| v.first()).cloned().unwrap(),
            message: map.get("message").and_then(|v| v.first()).cloned().unwrap(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
