use std::env;

use anyhow::{Context, Result};

use crate::{
    objects::{
        Tree,
        shared::{Object, ObjectType},
    },
    repository::Repository,
};

pub struct LsTreeCommand {
    tree: String,
    recursive: bool,
}

impl LsTreeCommand {
    pub fn new(tree: &str, recursive: bool) -> Self {
        Self {
            tree: tree.to_string(),
            recursive,
        }
    }

    pub fn execute(&self) -> Result<()> {
        let path = env::current_dir()?;
        let repo = Repository::find(path)?;

        Self::ls_tree(&self.tree, &repo, self.recursive, "")
    }

    fn ls_tree(sha: &str, repo: &Repository, recursive: bool, prefix: &str) -> Result<()> {
        let obj = repo.read_object(sha)?;
        let tree = obj
            .as_any()
            .downcast_ref::<Tree>()
            .context("Failed to downcast Tree")?;

        if !recursive {
            print!("{}", tree.pretty_print());
            return Ok(());
        }

        for row in &tree.rows {
            match row.object_type {
                ObjectType::Tree => Self::ls_tree(
                    &row.sha,
                    repo,
                    recursive,
                    &format!("{}{}/", prefix, row.filename),
                )?,
                _ => println!("{}", row.pretty_print(Some(prefix.to_string()))),
            }
        }
        Ok(())
    }
}
