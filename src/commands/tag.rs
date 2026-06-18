use std::env;

use anyhow::{Context, Ok, Result};

use crate::{
    commands::ShowRefCommand,
    objects::{Tag, shared::ObjectType},
    repository::Repository,
};

pub struct TagCommand {
    annotation: bool,
    name: Option<String>,
    object: String,
}

impl TagCommand {
    pub fn new(annotation: bool, name: &Option<String>, object: &str) -> Self {
        Self {
            annotation,
            name: name.clone(),
            object: object.to_string(),
        }
    }

    pub fn execute(&self) -> Result<()> {
        let path = env::current_dir()?;
        let repo = Repository::find(path)?;
        match &self.name {
            Some(_) => {
                self.create_tag(&repo)?;
            }
            None => {
                let tags_path = repo.gitdir.join("refs/tags");
                let refs = repo.ref_list(Some(tags_path))?;
                ShowRefCommand::show_ref(&refs, false, "");
            }
        };
        Ok(())
    }

    fn create_tag(&self, repo: &Repository) -> Result<()> {
        let sha = repo.find_sha(&self.object, None, false)?;

        match self.annotation {
            true => {
                let tag = Tag {
                    object: sha,
                    object_type: ObjectType::Commit,
                    tag: self.name.clone().unwrap(),
                    tagger: "VMR <merino.rodriguez.victor@gmail.com".into(),
                    message: "A tag generated with rust git version\n".into(),
                };
                let compressed_obj = repo.write_object(Box::new(tag))?;
                repo.create_ref(
                    &format!("tags/{}", self.name.clone().context("Invalid name")?),
                    &compressed_obj.sha,
                )?;
            }
            false => {
                repo.create_ref(
                    &format!("tags/{}", self.name.clone().context("Invalid name")?),
                    &sha,
                )?;
            }
        }
        Ok(())
    }
}
