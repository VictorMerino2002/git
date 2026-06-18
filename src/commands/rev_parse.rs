use std::env;

use anyhow::{Ok, Result};

use crate::{objects::shared::ObjectType, repository::Repository};

pub struct RevParse {
    object_type: ObjectType,
    name: String,
}

impl RevParse {
    pub fn new(object_type: &str, name: &str) -> Result<Self> {
        let object_type = ObjectType::try_from(object_type.to_string())?;

        Ok(Self {
            object_type,
            name: name.to_string(),
        })
    }

    pub fn execute(&self) -> Result<()> {
        let path = env::current_dir()?;
        let repo = Repository::find(path)?;

        let sha = repo.find_sha(&self.name, Some(&self.object_type), true)?;
        println!("{sha}");
        Ok(())
    }
}
