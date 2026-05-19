use anyhow::{Context, Result, bail};
use std::{fs, path::PathBuf};

use crate::config::Config;

pub struct Repository {
    pub worktree: PathBuf,
    pub gitdir: PathBuf,
}

impl Repository {
    pub fn init(path: PathBuf) -> Result<Self> {
        if path.exists() {
            if !path.is_dir() {
                bail!("{} is not a directory", path.display());
            }
            if path.join(".git").exists() {
                bail!("{} is already a git repository", path.display());
            }
        }
        fs::create_dir_all(&path).context("Failed to create repository directory")?;

        let gitdir = path.join(".git");
        fs::create_dir_all(gitdir.join("branches"))
            .context("Failed to create branches directory")?;
        fs::create_dir_all(gitdir.join("objects")).context("Failed to create objects directory")?;
        fs::create_dir_all(gitdir.join("refs/tags"))
            .context("Failed to create refs/tags directory")?;
        fs::create_dir_all(gitdir.join("refs/heads"))
            .context("Failed to create refs/heads directory")?;

        fs::write(
            gitdir.join("description"),
            "Unnamed repository; edit this file 'description' to name the repository.\n",
        )
        .context("Failed to write description file")?;

        fs::write(gitdir.join("HEAD"), "ref: refs/heads/master\n")
            .context("Failed to write HEAD file")?;
        fs::write(gitdir.join("config"), Config::default().to_string())
            .context("Failed to write config file")?;

        Ok(Self {
            worktree: path,
            gitdir,
        })
    }
}
