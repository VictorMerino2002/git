use anyhow::{Context, Result};
use std::{collections::HashMap, path::PathBuf};

pub struct IgnoreRule {
    pub pattern: Pattern,
    pub ignore: bool,
}

pub struct Pattern(pub String);

impl Pattern {
    pub fn match_with(&self, path: &str) -> bool {
        false
    }
}

pub struct GitIgnore {
    absolute_rules: Vec<IgnoreRule>,
    scoped_rules: HashMap<String, Vec<IgnoreRule>>,
}

impl GitIgnore {
    pub fn new() -> Self {
        Self {
            absolute_rules: vec![],
            scoped_rules: HashMap::new(),
        }
    }

    pub fn add_absolute_rules(&mut self, rules: Vec<IgnoreRule>) {
        self.absolute_rules.extend(rules);
    }

    pub fn add_scoped_rules(&mut self, dir: &str, rules: Vec<IgnoreRule>) {
        self.scoped_rules.insert(dir.into(), rules);
    }

    pub fn parse_line(line: &str) -> Option<IgnoreRule> {
        let raw = line.trim();
        if raw.is_empty() || raw.starts_with('#') {
            None
        } else if let Some(rest) = raw.strip_prefix('!') {
            Some(IgnoreRule {
                pattern: Pattern(rest.to_string()),
                ignore: false,
            })
        } else if let Some(rest) = raw.strip_prefix('\\') {
            Some(IgnoreRule {
                pattern: Pattern(rest.to_string()),
                ignore: true,
            })
        } else {
            Some(IgnoreRule {
                pattern: Pattern(raw.to_string()),
                ignore: true,
            })
        }
    }

    pub fn parse_lines(lines: &[&str]) -> Vec<IgnoreRule> {
        lines
            .iter()
            .filter_map(|line| Self::parse_line(line))
            .collect()
    }

    pub fn is_ignore_rule(rules: &[IgnoreRule], path: &str) -> Option<bool> {
        let mut result = None;
        for rule in rules {
            if rule.pattern.match_with(path) {
                result = Some(rule.ignore);
            }
        }
        result
    }

    pub fn is_ignore_scoped(&self, path: &str) -> Result<Option<bool>> {
        let mut parent = PathBuf::from(path)
            .parent()
            .context("Failed to get parent")?
            .to_path_buf();

        loop {
            let key = parent.to_string_lossy();
            if let Some(rules) = self.scoped_rules.get(key.as_ref()) {
                let result = Self::is_ignore_rule(rules, path);
                if result.is_some() {
                    return Ok(result);
                }
            }

            match parent.parent() {
                Some(p) => parent = p.to_path_buf(),
                None => break,
            }
        }

        Ok(None)
    }

    pub fn is_ignore_absolute(&self, path: &str) -> Option<bool> {
        Self::is_ignore_rule(&self.absolute_rules, path)
    }

    pub fn is_ignore(&self, path: &str) -> Result<Option<bool>> {
        if let Some(result) = self.is_ignore_absolute(path) {
            return Ok(Some(result));
        }

        self.is_ignore_scoped(path)
    }
}
