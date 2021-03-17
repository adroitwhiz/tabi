#[derive(Debug)]
pub enum AssetType {
    PNG,
    SVG,
    JPEG,
    WAV,
    MP3,
}

#[derive(Debug)]
pub struct Asset {
    pub data: Box<[u8]>,
    pub asset_type: AssetType,
    pub md5_digest: md5::Digest,
}

#[derive(Debug)]
pub struct Costume {
    pub asset: Asset,
    pub rotation_center: (f64, f64),
    pub name: String,
}
