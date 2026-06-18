use anyhow::Result;

use crate::{objects::shared::ObjectType, repository::Repository};

pub struct CatFileCommand {
    object_type: ObjectType,
    object: String,
}

impl CatFileCommand {
    pub fn new(object_type: &str, object: &str) -> Result<Self> {
        let object_type_enum = ObjectType::try_from(object_type.to_string())?;

        Ok(Self {
            object_type: object_type_enum,
            object: object.to_string(),
        })
    }

    pub fn execute(&self) -> Result<()> {
        let path = std::env::current_dir()?;
        let repo = Repository::find(path)?;
        let object_sha = repo.find_sha(&self.object, Some(&self.object_type), false)?;
        let object = repo.read_object(&object_sha)?;
        print!("{}", object.pretty_print());
        Ok(())
    }
}
