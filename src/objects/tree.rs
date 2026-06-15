use crate::objects::shared::{Object, ObjectType};

#[derive(Clone)]
pub struct Tree {
    pub rows: Vec<TreeRow>,
}

#[derive(Clone)]
pub struct TreeRow {
    pub mode: String,
    pub object_type: ObjectType,
    pub sha: String,
    pub filename: String,
}

impl TreeRow {
    pub fn new(mode: &str, sha: &str, filename: String) -> Self {
        let normalized_mode = if mode.len() == 5 {
            format!("0{}", mode)
        } else {
            mode.to_string()
        };

        let first_two = &normalized_mode[..2];
        let object_type = match first_two {
            "04" => ObjectType::Tree,
            "10" => ObjectType::Blob,
            "12" => ObjectType::Blob,
            "16" => ObjectType::Commit,
            _ => ObjectType::Blob,
        };

        Self {
            mode: normalized_mode,
            object_type,
            sha: sha.to_string(),
            filename,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(self.mode.as_bytes());
        result.push(b' ');
        result.extend_from_slice(self.filename.as_bytes());
        result.push(b'\0');

        for i in (0..self.sha.len()).step_by(2) {
            let byte =
                u8::from_str_radix(&self.sha[i..i + 2], 16).expect("Invalid hex character in SHA");
            result.push(byte);
        }

        result
    }

    pub fn deserialize(data: &[u8]) -> Self {
        let space_pos = data
            .iter()
            .position(|&b| b == b' ')
            .expect("Missing space in tree entry");

        let null_pos = data
            .iter()
            .position(|&b| b == b'\0')
            .expect("Missing null byte in tree entry");

        let mode_raw = std::str::from_utf8(&data[..space_pos]).expect("Invalid UTF-8 in mode");

        let mode = if mode_raw.len() == 5 {
            format!("0{}", mode_raw)
        } else {
            mode_raw.to_string()
        };

        let filename = std::str::from_utf8(&data[space_pos + 1..null_pos])
            .expect("Invalid UTF-8 in filename")
            .to_string();

        let sha_bytes = &data[null_pos + 1..null_pos + 21];
        let sha = sha_bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        Self::new(&mode, &sha, filename)
    }

    pub fn pretty_print(&self, parent_dir: Option<String>) -> String {
        let route = format!("{}{}", parent_dir.unwrap_or_default(), self.filename);
        format!("{} {} {}\t{}", self.mode, self.object_type, self.sha, route)
    }
}

impl Object for Tree {
    fn object_type(&self) -> ObjectType {
        ObjectType::Tree
    }

    fn serialize(&self) -> Vec<u8> {
        let mut sorted = self.rows.clone();
        sorted.sort_by_key(|r| {
            if r.object_type == ObjectType::Tree {
                format!("{}/", r.filename)
            } else {
                r.filename.clone()
            }
        });

        sorted.iter().flat_map(|row| row.serialize()).collect()
    }

    fn deserialize(data: &[u8]) -> Self {
        let mut rows = Vec::new();
        let mut start = 0;

        while start < data.len() {
            let null_offset = data[start..]
                .iter()
                .position(|&b| b == b'\0')
                .expect("Invalid tree object: missing null byte");

            let entry_end = start + null_offset + 1 + 20;

            let row = TreeRow::deserialize(&data[start..entry_end]);
            rows.push(row);
            start = entry_end;
        }

        Self { rows }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn pretty_print(&self) -> String {
        self.rows
            .iter()
            .map(|row| row.pretty_print(None))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
