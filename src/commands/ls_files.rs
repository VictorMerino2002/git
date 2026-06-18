use std::{env, fs};

use anyhow::Result;

use crate::{index::Index, repository::Repository};

pub struct LsFilesCommand {
    verbose: bool,
}

impl LsFilesCommand {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn execute(&self) -> Result<()> {
        let path = env::current_dir()?;
        let repo = Repository::find(path)?;

        let index_path = repo.gitdir.join("index");
        let index_bytes = fs::read(index_path)?;
        let index = Index::from_bytes(&index_bytes)?;

        if self.verbose {
            println!(
                "Index file format v{}, containing {} entries.",
                index.version,
                index.entries.len()
            );
            for e in index.entries {
                println!("{}", e.name);

                let entry_type = match e.mode_type {
                    0b0100 => "regular file (legacy)",
                    0b1000 => "regular file",
                    0b1010 => "symlink",
                    0b1110 => "git link",
                    _ => "unknown",
                };

                println!("  {} with perms: {}", entry_type, e.mode_perms);
                println!("  on blob: {}", e.sha);
                println!(
                    "  created: {}, modified: {}",
                    e.ctime.to_datetime()?,
                    e.mtime.to_datetime()?
                );
                println!("  device: {}, inode: {}", e.dev, e.ino);
                println!("  user: {}, group: {}", e.uid, e.gid);
                println!(
                    "  flags: stage={} assume_valid={}",
                    e.flag_stage, e.flag_assume_valid
                );
            }
        } else {
            for e in index.entries {
                println!("{}", e.name);
            }
        }

        Ok(())
    }
}
