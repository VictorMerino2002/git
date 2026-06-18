use anyhow::{Context, Result, bail};
use chrono::{DateTime, TimeZone, Utc};

pub struct Index {
    pub version: u32,
    pub entries: Vec<IndexEntry>,
}

pub struct IndexEntry {
    pub ctime: Timestamp,
    pub mtime: Timestamp,
    pub dev: u32,
    pub ino: u32,
    pub mode_type: u16,
    pub mode_perms: u16,
    pub uid: u32,
    pub gid: u32,
    pub fsize: u32,
    pub sha: String,
    pub flag_assume_valid: bool,
    pub flag_stage: u16,
    pub name: String,
}

pub struct Timestamp {
    pub seconds: u32,
    pub nanoseconds: u32,
}

impl Timestamp {
    pub fn to_datetime(&self) -> Result<DateTime<Utc>> {
        Utc.timestamp_opt(self.seconds as i64, self.nanoseconds)
            .single()
            .context("Invalid Timestamp")
    }
}

fn read_u32(data: &[u8], idx: &mut usize) -> Result<u32> {
    if *idx + 4 > data.len() {
        bail!("Unexpected end of data reading u32 at offset {}", idx);
    }
    let value = u32::from_be_bytes(data[*idx..*idx + 4].try_into()?);
    *idx += 4;
    Ok(value)
}

fn read_u16(data: &[u8], idx: &mut usize) -> Result<u16> {
    if *idx + 2 > data.len() {
        bail!("Unexpected end of data reading u16 at offset {}", idx);
    }
    let value = u16::from_be_bytes(data[*idx..*idx + 2].try_into()?);
    *idx += 2;
    Ok(value)
}

impl Index {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 12 {
            bail!("Index file too short to contain a valid header");
        }

        let signature = &bytes[0..4];
        if signature != b"DIRC" {
            bail!(
                "Invalid index signature: expected DIRC, got {:?}",
                signature
            );
        }

        let version = u32::from_be_bytes(bytes[4..8].try_into()?);
        if version != 2 {
            bail!(
                "Unsupported index version: {} (only version 2 is supported)",
                version
            );
        }

        let count = u32::from_be_bytes(bytes[8..12].try_into()?);

        let mut entries = Vec::with_capacity(count as usize);
        let mut idx: usize = 12;

        for _ in 0..count {
            let entry_start = idx;

            let ctime_s = read_u32(bytes, &mut idx)?;
            let ctime_ns = read_u32(bytes, &mut idx)?;

            let mtime_s = read_u32(bytes, &mut idx)?;
            let mtime_ns = read_u32(bytes, &mut idx)?;

            let dev = read_u32(bytes, &mut idx)?;
            let ino = read_u32(bytes, &mut idx)?;

            let mode_full = read_u32(bytes, &mut idx)?;
            let mode_raw = (mode_full & 0xFFFF) as u16;
            let mode_type = mode_raw >> 12;
            if !matches!(mode_type, 0b0100 | 0b1000 | 0b1010 | 0b1110) {
                bail!("Invalid mode type: {:04b}", mode_type);
            }
            let mode_perms = mode_raw & 0b0000_0001_1111_1111;

            let uid = read_u32(bytes, &mut idx)?;
            let gid = read_u32(bytes, &mut idx)?;
            let fsize = read_u32(bytes, &mut idx)?;

            if idx + 20 > bytes.len() {
                bail!("Unexpected end of data reading SHA at offset {}", idx);
            }
            let sha = bytes[idx..idx + 20]
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            idx += 20;

            let flags = read_u16(bytes, &mut idx)?;
            let flag_assume_valid = (flags & 0b1000_0000_0000_0000) != 0;
            let flag_extended = (flags & 0b0100_0000_0000_0000) != 0;
            if flag_extended {
                bail!("Extended flags are not supported");
            }
            let flag_stage = flags & 0b0011_0000_0000_0000;
            let name_length = (flags & 0b0000_1111_1111_1111) as usize;

            let raw_name = if name_length < 0xFFF {
                if idx + name_length >= bytes.len() {
                    bail!("Unexpected end of data reading name at offset {}", idx);
                }
                if bytes[idx + name_length] != 0x00 {
                    bail!(
                        "Expected null terminator after name at offset {}",
                        idx + name_length
                    );
                }
                let name_bytes = bytes[idx..idx + name_length].to_vec();
                idx += name_length + 1;
                name_bytes
            } else {
                eprintln!("Notice: Name is 0x{:X} bytes long.", name_length);
                let null_pos = bytes[idx..]
                    .iter()
                    .position(|&b| b == 0x00)
                    .map(|p| idx + p)
                    .ok_or_else(|| anyhow::anyhow!("No null terminator found for long name"))?;
                let name_bytes = bytes[idx..null_pos].to_vec();
                idx = null_pos + 1;
                name_bytes
            };

            let name = String::from_utf8(raw_name)
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in entry name: {}", e))?;

            let entry_size = idx - entry_start;
            let aligned_size = (entry_size + 7) & !7;
            idx = entry_start + aligned_size;

            entries.push(IndexEntry {
                ctime: Timestamp {
                    seconds: ctime_s,
                    nanoseconds: ctime_ns,
                },
                mtime: Timestamp {
                    seconds: mtime_s,
                    nanoseconds: mtime_ns,
                },
                dev,
                ino,
                mode_type,
                mode_perms,
                uid,
                gid,
                fsize,
                sha,
                flag_assume_valid,
                flag_stage,
                name,
            });
        }

        Ok(Index { version, entries })
    }
}
