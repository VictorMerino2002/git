use std::{env, path::PathBuf};

use anyhow::{Ok, Result};

use crate::repository::Repository;

pub struct CheckIgnoreCommand {
    paths: Vec<PathBuf>,
}

impl CheckIgnoreCommand {
    pub fn new(paths: &[String]) -> Self {
        let mut path_buffs = Vec::new();
        for p in paths {
            path_buffs.push(PathBuf::from(p));
        }
        Self { paths: path_buffs }
    }

    pub fn execute(&self) -> Result<()> {
        let repo_path = env::current_dir()?;
        let repo = Repository::find(repo_path)?;

        let git_ignore = repo.read_gitignore()?;

        for path in &self.paths {
            let is_ignored = git_ignore
                .is_ignore(&path.to_string_lossy())?
                .unwrap_or(false);
            if is_ignored {
                println!("{}", path.display());
            }
        }
        Ok(())
    }
}
