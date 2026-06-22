use std::{env, fs};

use anyhow::Result;

use crate::{
    index::Index,
    repository::Repository,
};

pub struct CommitCommand {
    pub message: String,
}

impl CommitCommand {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn execute(&self) -> Result<()> {
        let path = env::current_dir()?;
        let repo = Repository::find(path)?;

        let index_path = repo.gitdir.join("index");
        let index_bytes = fs::read(&index_path)?;
        let index = Index::from_bytes(&index_bytes)?;

        let tree = repo.tree_from_index(&index)?;

        let parent = match repo.find_sha("HEAD", None, true) {
            Ok(sha) => Some(sha),
            Err(_) => None,
        };

        let author = Repository::read_gitcfg()
            .ok()
            .and_then(|cfg| cfg.get_user())
            .unwrap_or_else(|| "Unknown <unknown>".to_string());

        let now = chrono::Utc::now();

        let commit = repo.commit_create(&tree, parent.as_deref(), &author, &now, &self.message)?;

        let active_branch = repo.get_active_branch()?;
        if let Some(branch) = active_branch {
            let ref_path = repo.gitdir.join("refs/heads").join(&branch);
            fs::write(ref_path, format!("{commit}\n"))?;
        } else {
            let head_path = repo.gitdir.join("HEAD");
            fs::write(head_path, format!("{commit}\n"))?;
        }

        println!("{commit}");
        Ok(())
    }
}
