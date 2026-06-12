use std::any::Any;

use crate::{
    objects::shared::{Object, ObjectType},
    utils::kvlm::{Kvlm, kvlm_parse, kvlm_serialize},
};

pub struct Commit {
    pub tree: String,
    pub parents: Vec<String>,
    pub author: String,
    pub committer: String,
    pub message: String,
}

impl Object for Commit {
    fn object_type(&self) -> ObjectType {
        ObjectType::Commit
    }

    fn serialize(&self) -> Vec<u8> {
        let mut map = Kvlm::new();
        map.insert("tree".into(), vec![self.tree.clone()]);
        map.insert("parent".into(), self.parents.clone());
        map.insert("author".into(), vec![self.author.clone()]);
        map.insert("committer".into(), vec![self.committer.clone()]);
        map.insert("message".into(), vec![self.message.clone()]);

        kvlm_serialize(&map).into_bytes()
    }

    fn deserialize(data: &[u8]) -> Self {
        let raw = std::str::from_utf8(data).expect("Invalid UTF-8");
        let map = kvlm_parse(raw);

        Self {
            tree: map
                .get("tree")
                .and_then(|v| v.first())
                .cloned()
                .unwrap_or_default(),
            parents: map.get("parent").cloned().unwrap_or_default(),
            author: map
                .get("author")
                .and_then(|v| v.first())
                .cloned()
                .unwrap_or_default(),
            committer: map
                .get("committer")
                .and_then(|v| v.first())
                .cloned()
                .unwrap_or_default(),
            message: map
                .get("message")
                .and_then(|v| v.first())
                .cloned()
                .unwrap_or_default(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
