use std::{collections::HashMap, fmt::Display};

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Config {
    pub repository_format_version: u32,
    pub file_mode: bool,
    pub bare: bool,
    pub raw: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn default() -> Self {
        Self {
            repository_format_version: 0,
            file_mode: true,
            bare: false,
            raw: HashMap::new(),
        }
    }

    pub fn from_str(content: &str) -> Result<Self> {
        let mut raw: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut current_section: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if line.starts_with('[') {
                let end = line.find(']').context("Malformed section header")?;
                let section_name = &line[1..end];
                current_section = Some(section_name.to_string());
                raw.entry(current_section.as_ref().unwrap().clone())
                    .or_default();
            } else if let Some(ref section) = current_section {
                if let Some(eq_pos) = line.find('=') {
                    let key = line[..eq_pos].trim().to_string();
                    let value = line[eq_pos + 1..].trim().to_string();
                    raw.get_mut(section).unwrap().insert(key, value);
                }
            }
        }

        Ok(Self::from_raw(raw))
    }

    pub fn get_user(&self) -> Option<String> {
        let user = self.raw.get("user")?;
        let name = user.get("name")?;
        let email = user.get("email")?;
        Some(format!("{name} <{email}>"))
    }

    pub fn from_raw(raw: HashMap<String, HashMap<String, String>>) -> Self {
        let repository_format_version = raw
            .get("core")
            .and_then(|s| s.get("repositoryformatversion"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let file_mode = raw
            .get("core")
            .and_then(|s| s.get("filemode"))
            .map(|v| v == "true")
            .unwrap_or(true);

        let bare = raw
            .get("core")
            .and_then(|s| s.get("bare"))
            .map(|v| v == "true")
            .unwrap_or(false);

        Self {
            repository_format_version,
            file_mode,
            bare,
            raw,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[core]\n\trepositoryformatversion = {}\n\tfilemode = {}\n\tbare = {}",
            self.repository_format_version, self.file_mode, self.bare
        )
    }
}
