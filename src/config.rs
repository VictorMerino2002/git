use std::fmt::Display;

pub struct Config {
    repository_format_version: u32,
    file_mode: bool,
    bare: bool,
}

impl Config {
    pub fn default() -> Self {
        Self {
            repository_format_version: 0,
            file_mode: true,
            bare: false,
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
