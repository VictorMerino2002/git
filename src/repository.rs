use anyhow::{Context, Ok, Result, bail};
use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    config::Config,
    objects::{
        Blob, Commit, Tree,
        shared::{CompressedObject, Object, ObjectType},
    },
    utils::{sha1, zlib},
};

pub struct Repository {
    pub worktree: PathBuf,
    pub gitdir: PathBuf,
}

#[derive(Debug)]
pub enum RefValue {
    Sha(String),
    Nested(HashMap<String, RefValue>),
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

    pub fn find(path: PathBuf) -> Result<Self> {
        let gitdir = path.join(".git");
        if gitdir.exists() {
            return Ok(Self {
                worktree: path,
                gitdir,
            });
        }
        let parent = path.parent().context("Failed to get parent directory")?;
        if parent == path {
            bail!("No git repository found");
        }

        Self::find(parent.to_path_buf())
    }

    pub fn read_object(&self, sha: &str) -> Result<Box<dyn Object>> {
        let object_path = self.gitdir.join("objects").join(&sha[0..2]).join(&sha[2..]);
        if !object_path.exists() {
            bail!("Object {} not found", sha);
        }
        let data = fs::read(&object_path).context("Failed to read object file")?;
        let raw = zlib::decompress(&data)?;

        let space = raw
            .iter()
            .position(|&b| b == b' ')
            .context(format!("Malformed object {sha}"))?;

        let object_type = &raw[..space];

        let null = raw
            .iter()
            .position(|&b| b == b'\0')
            .context(format!("Malformed object {sha}"))?;

        let size: usize = std::str::from_utf8(&raw[space + 1..null])
            .context("Invalid size encoding")?
            .parse()
            .context("Invalid size number")?;

        if size != raw.len() - null - 1 {
            bail!("Malformed object {sha}: bad length");
        }

        let content = &raw[null + 1..];

        let object: Box<dyn Object> =
            match std::str::from_utf8(object_type).context("Invalid object type")? {
                "blob" => Box::new(Blob::deserialize(content)),
                "commit" => Box::new(Commit::deserialize(content)),
                "tree" => Box::new(Tree::deserialize(content)),
                t => bail!("Unknown object type: {t}"),
            };
        Ok(object)
    }

    pub fn compress_object(obj: Box<dyn Object>) -> Result<CompressedObject> {
        let content = obj.serialize();
        let data: Vec<u8> = [
            obj.object_type().to_string().as_bytes(),
            b" ",
            content.len().to_string().as_bytes(),
            b"\0",
            &content,
        ]
        .concat();

        let sha = sha1::sha(&data);
        let data = zlib::compress(&data).context("Failed to compress object")?;

        Ok(CompressedObject { sha, data })
    }

    pub fn write_object(&self, obj: Box<dyn Object>) -> Result<CompressedObject> {
        let compressed_obj = Self::compress_object(obj)?;

        let path = self.gitdir.join(format!(
            "objects/{}/{}",
            &compressed_obj.sha[..2],
            &compressed_obj.sha[2..]
        ));

        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap()).context("Failed to create object dir")?;
            fs::write(path, &compressed_obj.data).context("Failed to write obj")?;
        }

        Ok(compressed_obj)
    }

    pub fn find_sha(&self, name: &str, object_type: Option<&ObjectType>) -> Result<String> {
        Ok(name.to_string())
    }

    pub fn ref_resolve(&self, reference: &str) -> Option<String> {
        let path = self.gitdir.join(reference);
        if !path.is_file() {
            return None;
        }
        let content = fs::read_to_string(path).ok()?;
        if let Some(new_reference) = content.strip_prefix("ref: ") {
            return self.ref_resolve(new_reference.trim());
        }
        Some(content.trim().to_string())
    }

    pub fn ref_list(&self, path_opt: Option<PathBuf>) -> Result<(HashMap<String, RefValue>)> {
        let path = path_opt.unwrap_or_else(|| self.gitdir.join("refs"));

        let mut entries = fs::read_dir(&path)?
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>();

        entries.sort_by_key(|e| e.file_name());
        let mut ret = HashMap::new();

        for e in entries {
            let entry_path = path.join(e.file_name());
            let name = e.file_name().to_string_lossy().to_string();

            if entry_path.is_dir() {
                ret.insert(name, RefValue::Nested(self.ref_list(Some(entry_path))?));
            } else {
                let relative = entry_path
                    .strip_prefix(&self.gitdir)
                    .context("Ref path outsite gitdir")?;

                if let Some(sha) = self.ref_resolve(&relative.to_string_lossy()) {
                    ret.insert(name, RefValue::Sha(sha));
                }
            }
        }

        Ok(ret)
    }

    pub fn create_ref(&self, ref_name: &str, sha: &str) -> Result<()> {
        let path = self.gitdir.join("refs").join(ref_name);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, format!("{sha}\n"))?;
        Ok(())
    }
}
