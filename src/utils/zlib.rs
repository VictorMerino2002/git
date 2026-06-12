use anyhow::{Context, Result};
use flate2::{
    Compression,
    read::{ZlibDecoder, ZlibEncoder},
};
use std::io::Read;

pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(data, Compression::default());
    let mut result = Vec::new();
    encoder
        .read_to_end(&mut result)
        .context("Failed to encode data")?;
    Ok(result)
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder
        .read_to_end(&mut result)
        .context("Failed to decode data")?;
    Ok(result)
}
