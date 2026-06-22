use std::env;

use anyhow::Result;

use crate::repository::Repository;

pub struct RmCommand {
    pub paths: Vec<String>,
}

impl RmCommand {
    pub fn new(paths: &[String]) -> Self {
        RmCommand {
            paths: paths.to_vec(),
        }
    }

    pub fn execute(&self) -> Result<()> {
        let path = env::current_dir()?;
        let repo = Repository::find(path)?;
        repo.rm(&self.paths, true, false)
    }
}
