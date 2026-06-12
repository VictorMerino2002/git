use sha1::{Digest, Sha1};

pub fn hash(data: &[u8]) -> impl AsRef<[u8]> {
    Sha1::digest(data)
}

pub fn sha(data: &[u8]) -> String {
    let hash = Sha1::digest(data);
    hex::encode(hash)
}
