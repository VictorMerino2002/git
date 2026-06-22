use std::{env, fs};

use anyhow::Result;

use crate::{index::Index, repository::Repository, utils::sha1};

pub struct StatusCommand {
    pub repo: Repository,
    pub index: Index,
}

impl StatusCommand {
    pub fn new() -> Result<Self> {
        let path = env::current_dir()?;
        let repo = Repository::find(path)?;

        let index_path = repo.gitdir.join("index");
        let bytes = fs::read(index_path)?;
        let index = Index::from_bytes(&bytes)?;

        Ok(Self { repo, index })
    }

    pub fn execute(&self) -> Result<()> {
        self.print_status_branch()?;
        self.print_status_head_index()?;
        self.print_status_index_worktree()?;
        Ok(())
    }

    fn print_status_branch(&self) -> Result<()> {
        let branch_opt = self.repo.get_active_branch()?;

        match branch_opt {
            Some(branch) => println!("On branch {}", branch),
            None => {
                let reference = self.repo.find_sha("HEAD", None, false)?;
                println!("HEAD detached at {}", reference)
            }
        };

        Ok(())
    }

    fn print_status_head_index(&self) -> Result<()> {
        println!("Changes to be committed:");

        let mut head = self.repo.tree_to_dict("HEAD", "")?;
        for entry in &self.index.entries {
            if head.contains_key(&entry.name) {
                if head.get(&entry.name).unwrap() != &entry.sha {
                    println!("  modified: {}", entry.name);
                }
                head.remove(&entry.name);
            } else {
                println!("  added:  {}", entry.name);
            }
        }

        for entry in head.keys() {
            println!("  deleted: {}", entry);
        }
        Ok(())
    }

    fn print_status_index_worktree(&self) -> Result<()> {
        println!("Changes not staged for commit:");

        let gitignore = self.repo.read_gitignore()?;
        let gitdir_prefix = format!("{}/", self.repo.gitdir.to_string_lossy());

        let mut all_files: Vec<String> = Vec::new();
        let mut dirs_to_visit = vec![self.repo.worktree.clone()];

        while let Some(current_dir) = dirs_to_visit.pop() {
            let dir_str = current_dir.to_string_lossy();

            if current_dir == self.repo.gitdir || dir_str.starts_with(&gitdir_prefix) {
                continue;
            }

            for entry in fs::read_dir(&current_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    dirs_to_visit.push(path);
                } else {
                    let rel = path
                        .strip_prefix(&self.repo.worktree)?
                        .to_string_lossy()
                        .into_owned();
                    all_files.push(rel);
                }
            }
        }

        for entry in &self.index.entries {
            let full_path = self.repo.worktree.join(&entry.name);

            if !full_path.exists() {
                println!("  deleted: {}", entry.name);
            } else {
                let meta = fs::metadata(&full_path)?;

                let ctime_ns =
                    entry.ctime.seconds as u64 * 1_000_000_000 + entry.ctime.nanoseconds as u64;
                let mtime_ns =
                    entry.mtime.seconds as u64 * 1_000_000_000 + entry.mtime.nanoseconds as u64;

                let meta_ctime_ns = meta
                    .created()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);
                let meta_mtime_ns = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);

                if meta_ctime_ns != ctime_ns || meta_mtime_ns != mtime_ns {
                    let content = fs::read(&full_path)?;
                    let header = format!("blob {}\0", content.len());
                    let full_data = [header.as_bytes(), &content].concat();
                    let new_sha = sha1::sha(&full_data);

                    if new_sha != entry.sha {
                        println!("  modified: {}", entry.name);
                    }
                }
            }

            all_files.retain(|f| f != &entry.name);
        }

        println!();
        println!("Untracked files:");

        for f in &all_files {
            let is_ignored = gitignore.is_ignore(f)?.unwrap_or(false);
            if !is_ignored {
                println!("  {}", f);
            }
        }

        Ok(())
    }
}
