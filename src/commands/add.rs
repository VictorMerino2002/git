use std::env;

use anyhow::Result;

use crate::repository::Repository;

pub struct AddCommand {
    pub paths: Vec<String>,
}

impl AddCommand {
    pub fn new(paths: &[String]) -> Self {
        AddCommand {
            paths: paths.to_vec(),
        }
    }

    pub fn execute(&self) -> Result<()> {
        let path = env::current_dir()?;
        let repo = Repository::find(path)?;
        repo.add(&self.paths)
    }
}
