use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    objects::{Blob, Commit, Tree, shared::ObjectType},
    repository::Repository,
};

pub struct Checkout {
    commit: String,
    path: PathBuf,
}

impl Checkout {
    pub fn new(commit: &str, path: &str) -> Self {
        Self {
            commit: commit.to_string(),
            path: PathBuf::from(path),
        }
    }

    pub fn execute(&self) -> Result<()> {
        let path = env::current_dir()?;
        let rel_path = path.join(&self.path);
        let repo = Repository::find(path)?;

        let obj = repo.read_object(&self.commit)?;

        match obj.object_type() {
            ObjectType::Tree => {
                let tree = obj
                    .as_any()
                    .downcast_ref::<Tree>()
                    .context("Failed to downcast Tree")?;
                Self::tree_checkout(&repo, tree, &rel_path)?;
            }
            ObjectType::Commit => {
                let commit = obj
                    .as_any()
                    .downcast_ref::<Commit>()
                    .context("Failed to downcast Commit")?;
                let obj_tree = repo.read_object(&commit.tree)?;
                let tree = obj_tree
                    .as_any()
                    .downcast_ref::<Tree>()
                    .context("Failed to downcast Tree")?;
                Self::tree_checkout(&repo, tree, &rel_path)?;
            }
            _ => {}
        };

        Ok(())
    }

    fn tree_checkout(repo: &Repository, tree: &Tree, path: &PathBuf) -> Result<()> {
        for row in &tree.rows {
            let obj = repo.read_object(&row.sha)?;
            let destination = path.join(PathBuf::from(&row.filename));

            match obj.object_type() {
                ObjectType::Tree => {
                    let tree = obj
                        .as_any()
                        .downcast_ref::<Tree>()
                        .context("Failed to downcast Tree")?;
                    Self::tree_checkout(repo, tree, &destination)?;
                }
                ObjectType::Blob => {
                    let blob = obj
                        .as_any()
                        .downcast_ref::<Blob>()
                        .context("Failed to downcast Blob")?;
                    fs::write(destination, &blob.data)?;
                }
                _ => {}
            };
        }
        Ok(())
    }
}
