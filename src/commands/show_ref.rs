use std::{collections::HashMap, env};

use anyhow::{Ok, Result};

use crate::repository::{RefValue, Repository};

pub struct ShowRefCommand;

impl ShowRefCommand {
    pub fn execute() -> Result<()> {
        let path = env::current_dir()?;
        let repo = Repository::find(path)?;
        let refs = repo.ref_list(None)?;

        Self::show_ref(&refs, true, "refs");
        Ok(())
    }

    fn show_ref(refs: &HashMap<String, RefValue>, with_hash: bool, prefix: &str) {
        for (k, v) in refs {
            let full_key = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{}/{}", prefix, k)
            };

            match v {
                RefValue::Sha(sha) => {
                    if with_hash {
                        println!("{} {}", sha, full_key);
                    } else {
                        println!("{}", full_key);
                    }
                }
                RefValue::Nested(nested) => {
                    Self::show_ref(nested, with_hash, &full_key);
                }
            }
        }
    }
}
