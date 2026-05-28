use std::{fs, path::PathBuf};

use anyhow::Result;

use crate::{
    objects::{
        Blob,
        shared::{Object, ObjectType},
    },
    repository::Repository,
};

pub struct HashObjectCommand {
    object_type: ObjectType,
    write: bool,
    path: PathBuf,
}

impl HashObjectCommand {
    pub fn new(object_type: &str, write: bool, path: &str) -> Result<Self> {
        let object_type = ObjectType::try_from(object_type.to_string())?;
        let path = PathBuf::from(path);
        if !path.exists() {
            anyhow::bail!("File does not exist: {}", path.display());
        }
        Ok(Self {
            object_type,
            write,
            path,
        })
    }

    pub fn execute(&self) -> Result<()> {
        let data = fs::read(&self.path)?;
        let object: Box<dyn Object> = match self.object_type {
            ObjectType::Blob => Box::new(Blob { data }),
            _ => anyhow::bail!("Unsupported object type: {}", self.object_type),
        };
        if self.write {
            let path = std::env::current_dir()?;
            let repo = Repository::find(path)?;
            let _ = repo.write_object(object)?;
        } else {
            let compressed_obj = Repository::compress_object(object)?;
            println!("{}", compressed_obj.sha);
        }

        Ok(())
    }
}
