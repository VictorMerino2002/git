use crate::{
    objects::{Commit, shared::ObjectType},
    repository::Repository,
};
use anyhow::{Context, Result, bail};
use std::collections::{HashSet, VecDeque};

pub struct LogCommand {
    commit: String,
}

impl LogCommand {
    pub fn new(commit: &str) -> Result<Self> {
        Ok(Self {
            commit: commit.to_string(),
        })
    }

    pub fn execute(&self) -> Result<()> {
        let path = std::env::current_dir()?;
        let repo = Repository::find(path)?;

        let mut seen = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(self.commit.clone());

        while let Some(current_commit) = queue.pop_front() {
            if seen.contains(&current_commit) {
                continue;
            }
            seen.insert(current_commit.clone());

            let object = repo.read_object(&current_commit)?;
            match object.object_type() {
                ObjectType::Commit => {}
                _ => bail!("Object {} is not a commit", current_commit),
            };

            let commit = object
                .as_any()
                .downcast_ref::<Commit>()
                .context("Failed to downcast object to commit")?;

            println!("commit {}", current_commit);
            println!("Author: {}", commit.author);
            println!();
            println!("    {}", commit.message.trim());
            println!();

            for parent in &commit.parents {
                if !seen.contains(parent.as_str()) {
                    queue.push_back(parent.clone());
                }
            }
        }

        Ok(())
    }
}
