use std::fmt;

#[derive(Debug)]
pub enum AssetType {
    PNG,
    SVG,
    JPEG,
    WAV,
    MP3,
}

pub struct Asset {
    pub data: Box<[u8]>,
    pub asset_type: AssetType,
    pub md5_digest: md5::Digest,
}

impl fmt::Debug for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Asset")
            .field("asset_type", &self.asset_type)
            .field("md5_digest", &self.md5_digest)
            .finish()
    }
}
