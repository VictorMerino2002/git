use anyhow::{Context, Ok, Result, bail};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
};

use crate::{
    config::Config,
    git_ignore::{GitIgnore, IgnoreRule},
    index::{Index, IndexEntry, Timestamp},
    objects::{
        Blob, Commit, Tag, Tree, TreeRow,
        shared::{CompressedObject, Object, ObjectType},
    },
    utils::{sha1, zlib},
};

enum EntryOrTree<'a> {
    Entry(&'a IndexEntry),
    TreeItem((String, String)),
}

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
            &compressed_obj.sha.chars().take(2).collect::<String>(),
            &compressed_obj.sha.chars().skip(2).collect::<String>()
        ));

        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap()).context("Failed to create object dir")?;
            fs::write(path, &compressed_obj.data).context("Failed to write obj")?;
        }

        Ok(compressed_obj)
    }

    pub fn find_sha(
        &self,
        name: &str,
        object_type: Option<&ObjectType>,
        follow: bool,
    ) -> Result<String> {
        let sha_list = self.object_resolve(name)?;

        if sha_list.is_empty() {
            bail!("No such reference");
        }

        if sha_list.len() > 1 {
            let candidates_display = sha_list.join("\n - ");
            bail!("Ambiguous reference {name}: Candidates are:\n - {candidates_display}")
        }

        let mut sha = sha_list[0].clone();

        if object_type.is_none() {
            return Ok(sha);
        }

        loop {
            let obj = self.read_object(&sha)?;

            if &obj.object_type() == object_type.unwrap() {
                return Ok(sha);
            }

            if !follow {
                bail!("Object Not found");
            }

            match obj.object_type() {
                ObjectType::Tag => {
                    let tag = obj
                        .as_any()
                        .downcast_ref::<Tag>()
                        .context("Failed to downcast Tag")?;
                    sha = tag.object.clone();
                }
                ObjectType::Commit => {
                    if object_type.unwrap() != &ObjectType::Tree {
                        bail!("Object Not found");
                    }
                    let commit = obj
                        .as_any()
                        .downcast_ref::<Commit>()
                        .context("Failed to downcast Commit")?;
                    sha = commit.tree.clone();
                }
                _ => bail!("Object not found"),
            };
        }
    }

    pub fn object_resolve(&self, name: &str) -> Result<Vec<String>> {
        if name == "HEAD" {
            let head = self.ref_resolve("HEAD").context("Failed to resolve HEAD")?;
            return Ok(vec![head]);
        }

        let hash_pattern = Regex::new("^[0-9A-Fa-f]{4,40}$")?;
        let mut candidates = vec![];

        if hash_pattern.is_match(name) {
            let name_lower = name.to_lowercase();
            let prefix = name_lower.chars().take(2).collect::<String>();
            let path = self.gitdir.join("objects").join(&prefix);

            if path.exists() && path.is_dir() {
                let rem = name_lower.chars().skip(2).collect::<String>();
                for f in fs::read_dir(path)? {
                    let file_name = f?.file_name().to_string_lossy().to_string();
                    if file_name.starts_with(&rem) {
                        candidates.push(format!("{}{}", prefix, file_name));
                    }
                }
            }
        }
        let as_tag = self.ref_resolve(&format!("refs/tags/{}", name));
        let as_branch = self.ref_resolve(&format!("refs/heads/{}", name));
        let as_remote_branch = self.ref_resolve(&format!("refs/remotes/{}", name));

        if let Some(tag_sha) = as_tag {
            candidates.push(tag_sha);
        }

        if let Some(branch_sha) = as_branch {
            candidates.push(branch_sha);
        }

        if let Some(remote_branch_sha) = as_remote_branch {
            candidates.push(remote_branch_sha);
        }

        Ok(candidates)
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

    pub fn read_gitcfg() -> Result<Config> {
        let xdg_config_home = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| "~".to_string());
            format!("{}/.config", home)
        });

        let config_paths = vec![
            PathBuf::from(&xdg_config_home).join("git/config"),
            PathBuf::from(env::var("HOME").unwrap_or_else(|_| "~".to_string())).join(".gitconfig"),
        ];

        let mut raw: HashMap<String, HashMap<String, String>> = HashMap::new();
        for path in &config_paths {
            if path.exists() {
                let content = fs::read_to_string(path)
                    .context(format!("Failed to read {}", path.display()))?;
                if let std::result::Result::Ok(cfg) = Config::from_str(&content) {
                    for (section, keys) in cfg.raw {
                        raw.entry(section).or_default().extend(keys);
                    }
                }
            }
        }

        Ok(Config::from_raw(raw))
    }

    pub fn read_gitignore(&self) -> Result<GitIgnore> {
        let mut git_ignore = GitIgnore::new();

        let local_cfg_path = self.gitdir.join("info/exclude");
        if local_cfg_path.exists() {
            let content = fs::read_to_string(local_cfg_path)?;
            let lines = content.lines().collect::<Vec<&str>>();
            let rules = GitIgnore::parse_lines(&lines);
            git_ignore.add_absolute_rules(rules);
        }

        let cfg_home = env::var("XDG_CONFIG_HOME")
            .or_else(|_| env::var("HOME").map(|h| format!("{}/.config", h)))?;
        let global_cfg_path = PathBuf::from(cfg_home).join("git/ignore");
        if global_cfg_path.exists() {
            let content = fs::read_to_string(global_cfg_path)?;
            let lines = content.lines().collect::<Vec<&str>>();
            let rules = GitIgnore::parse_lines(&lines);
            git_ignore.add_absolute_rules(rules);
        }

        self.load_worktree_gitignores(&mut git_ignore);
        self.load_index_gitignores(&mut git_ignore)?;

        Ok(git_ignore)
    }

    fn load_worktree_gitignores(&self, git_ignore: &mut GitIgnore) {
        let gitignore_path = self.worktree.join(".gitignore");
        if gitignore_path.exists() {
            if let Some(content) = fs::read_to_string(gitignore_path).ok() {
                let lines = content.lines().collect::<Vec<&str>>();
                let rules = GitIgnore::parse_lines(&lines);
                git_ignore.add_scoped_rules("", rules);
            }
        }
    }

    fn load_index_gitignores(&self, git_ignore: &mut GitIgnore) -> Result<()> {
        let index_path = self.gitdir.join("index");
        let index_bytes = match fs::read(index_path) {
            std::result::Result::Ok(bytes) => bytes,
            std::result::Result::Err(_) => return std::result::Result::Ok(()),
        };
        let index = match Index::from_bytes(&index_bytes) {
            std::result::Result::Ok(idx) => idx,
            std::result::Result::Err(_) => return std::result::Result::Ok(()),
        };

        for entry in index.entries {
            if entry.name == ".gitignore" || entry.name.ends_with("/.gitignore") {
                let entry_path = PathBuf::from(&entry.name);
                let dir_name = entry_path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                if let Some(obj) = self.read_object(&entry.sha).ok() {
                    if let Some(blob) = obj.as_any().downcast_ref::<Blob>() {
                        let content = String::from_utf8_lossy(&blob.data).into_owned();
                        let lines = content.lines().collect::<Vec<&str>>();
                        let rules = GitIgnore::parse_lines(&lines);
                        git_ignore.add_scoped_rules(&dir_name, rules);
                    }
                }
            }
        }

        std::result::Result::Ok(())
    }

    pub fn get_active_branch(&self) -> Result<Option<String>> {
        let head_file = self.gitdir.join("HEAD");
        let head = fs::read_to_string(head_file)?;
        if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
            Ok(Some(branch.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn tree_to_dict(&self, reference: &str, prefix: &str) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        let tree_sha = self.find_sha(reference, Some(&ObjectType::Tree), false)?;
        let obj = self.read_object(&tree_sha)?;
        let tree_obj = obj
            .as_any()
            .downcast_ref::<Tree>()
            .context("Failed to downcast Tree")?;
        for row in &tree_obj.rows {
            let full_path = PathBuf::from(prefix).join(&row.filename);
            let full_path_str = full_path.to_string_lossy().into_owned();

            match row.object_type {
                ObjectType::Tree => {
                    let subtree = self.tree_to_dict(&row.sha, &full_path_str)?;
                    map.extend(subtree);
                }
                _ => {
                    map.insert(full_path_str, row.sha.clone());
                }
            };
        }
        Ok(map)
    }

    pub fn rm(&self, paths: &[String], delete: bool, skip_missing: bool) -> Result<()> {
        let index_path = self.gitdir.join("index");
        let index_bytes = fs::read(&index_path).context("Failed to read index file")?;
        let mut index = Index::from_bytes(&index_bytes)?;

        let worktree = self.worktree.to_string_lossy().to_string() + "/";

        let mut abspaths: HashSet<String> = HashSet::new();
        for path in paths {
            let abs_path = std::path::absolute(path)
                .context("Failed to resolve absolute path")?
                .to_string_lossy()
                .to_string();
            if abs_path.starts_with(&worktree) {
                abspaths.insert(abs_path);
            } else {
                bail!("Cannot remove paths outside of worktree: {paths:?}");
            }
        }

        let mut remove = Vec::new();

        index.entries = index
            .entries
            .into_iter()
            .filter(|e| {
                let full_path = self.worktree.join(&e.name);
                let full_path_str = full_path.to_string_lossy().to_string();

                if abspaths.contains(&full_path_str) {
                    remove.push(full_path);
                    abspaths.remove(&full_path_str);
                    false
                } else {
                    true
                }
            })
            .collect();

        if !abspaths.is_empty() && !skip_missing {
            bail!(
                "Cannot remove paths not in the index: {:?}",
                abspaths.iter().collect::<Vec<_>>()
            );
        }

        if delete {
            for path in &remove {
                fs::remove_file(path).context("Failed to remove file")?;
            }
        }
        let new_index_bytes = index.to_bytes()?;
        fs::write(&index_path, new_index_bytes).context("Failed to write index file")?;

        Ok(())
    }

    pub fn commit_create(
        &self,
        tree: &str,
        parent: Option<&str>,
        author: &str,
        timestamp: &chrono::DateTime<chrono::Utc>,
        message: &str,
    ) -> Result<String> {
        let tz = "+0000";
        let author_line = format!("{} {} {}", author, timestamp.timestamp(), tz);

        let commit = Commit {
            tree: tree.to_string(),
            parents: parent.map(|p| vec![p.to_string()]).unwrap_or_default(),
            author: author_line.clone(),
            committer: author_line,
            message: message.trim().to_string() + "\n",
        };

        let obj: Box<dyn Object> = Box::new(commit);
        let compressed = self.write_object(obj)?;
        Ok(compressed.sha)
    }

    pub fn tree_from_index(&self, index: &Index) -> Result<String> {
        let mut contents: HashMap<String, Vec<EntryOrTree>> = HashMap::new();
        contents.entry("".to_string()).or_default();

        for entry in &index.entries {
            let dirname = Path::new(&entry.name)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let mut key = dirname.clone();
            while key != "" {
                contents.entry(key.clone()).or_default();
                key = Path::new(&key)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
            }

            contents
                .entry(dirname)
                .or_default()
                .push(EntryOrTree::Entry(entry));
        }

        let mut sorted_paths: Vec<String> = contents.keys().cloned().collect();
        sorted_paths.sort_by(|a, b| b.len().cmp(&a.len()));

        let mut sha = String::new();

        for path in &sorted_paths {
            let mut tree = Tree { rows: Vec::new() };

            for item in &contents[path] {
                match item {
                    EntryOrTree::Entry(entry) => {
                        let mode = format!("{:02o}{:04o}", entry.mode_type, entry.mode_perms);
                        let leaf = TreeRow::new(
                            &mode,
                            &entry.sha,
                            Path::new(&entry.name)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default(),
                        );
                        tree.rows.push(leaf);
                    }
                    EntryOrTree::TreeItem((base_name, tree_sha)) => {
                        let leaf = TreeRow::new("040000", tree_sha, base_name.clone());
                        tree.rows.push(leaf);
                    }
                }
            }

            let obj: Box<dyn Object> = Box::new(tree);
            let compressed = self.write_object(obj)?;
            sha = compressed.sha;

            let parent = Path::new(path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let base = Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            contents
                .entry(parent)
                .or_default()
                .push(EntryOrTree::TreeItem((base, sha.clone())));
        }

        Ok(sha)
    }

    pub fn add(&self, paths: &[String]) -> Result<()> {
        self.rm(paths, false, true)?;

        let worktree = self.worktree.to_string_lossy().to_string() + "/";

        let mut clean_paths = Vec::new();
        for path in paths {
            let abs_path = std::path::absolute(path)
                .context("Failed to resolve absolute path")?
                .to_string_lossy()
                .to_string();
            if !abs_path.starts_with(&worktree) || !std::path::Path::new(&abs_path).is_file() {
                bail!("Not a file, or outside the worktree: {paths:?}");
            }
            let relpath = std::path::Path::new(&abs_path)
                .strip_prefix(&self.worktree)
                .context("Failed to compute relative path")?
                .to_string_lossy()
                .to_string();
            clean_paths.push((abs_path, relpath));
        }

        let index_path = self.gitdir.join("index");
        let index_bytes = fs::read(&index_path).context("Failed to read index file")?;
        let mut index = Index::from_bytes(&index_bytes)?;

        for (abspath, relpath) in &clean_paths {
            let data = fs::read(abspath).context("Failed to read file")?;
            let object: Box<dyn Object> = Box::new(Blob { data });
            let compressed = self.write_object(object)?;

            let stat = fs::metadata(abspath).context("Failed to get file metadata")?;

            let mtime = stat.modified().context("Failed to get mtime")?;
            let duration_since_epoch = mtime
                .duration_since(std::time::UNIX_EPOCH)
                .context("File time is before Unix epoch")?;

            let entry = IndexEntry {
                ctime: Timestamp {
                    seconds: 0,
                    nanoseconds: 0,
                },
                mtime: Timestamp {
                    seconds: duration_since_epoch.as_secs() as u32,
                    nanoseconds: duration_since_epoch.subsec_nanos(),
                },
                dev: 0,
                ino: 0,
                mode_type: 0b1000,
                mode_perms: 0o644,
                uid: 0,
                gid: 0,
                fsize: stat.len() as u32,
                sha: compressed.sha,
                flag_assume_valid: false,
                flag_stage: 0,
                name: relpath.clone(),
            };
            index.entries.push(entry);
        }

        let new_index_bytes = index.to_bytes()?;
        fs::write(&index_path, new_index_bytes).context("Failed to write index file")?;

        Ok(())
    }

    pub fn write_index(&self, index: &Index) -> Result<()> {
        let index_bytes = index.to_bytes()?;
        let index_path = self.gitdir.join("index");
        fs::write(index_path, index_bytes).context("Failed to write index file")?;
        Ok(())
    }
}
