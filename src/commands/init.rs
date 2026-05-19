use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

use crate::repository::Repository;

pub struct InitCommand {
    pub path: String,
}

impl InitCommand {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn execute(&self) -> Result<()> {
        let path = PathBuf::from(&self.path);

        let repo = Repository::init(path)?;

        let abs_path = fs::canonicalize(&repo.gitdir).context("Failed to get absolute path")?;
        println!("Initialized empty git repository in {}", abs_path.display());
        Ok(())
    }
}
