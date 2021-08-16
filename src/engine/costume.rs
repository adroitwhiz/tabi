use std::rc::Rc;

use crate::{data::asset::{Asset, AssetType}, renderer::{renderer::Renderer, skin::Skin}};

#[derive(Debug)]
pub struct CostumeAsset {
    pub asset: Asset,
    pub rotation_center: (f64, f64),
    pub name: String,
}

impl CostumeAsset {
    pub fn load (self, renderer: &mut Renderer) -> Costume {
        let skin = match self.asset.asset_type {
            AssetType::SVG => {
                renderer.create_svg_skin(&self.asset.data, self.rotation_center)
            },
            AssetType::PNG => {
                unimplemented!()
            },
            AssetType::JPEG => {
                unimplemented!()
            },
            AssetType::MP3 | AssetType::WAV => {
                panic!("Costume given non-image asset type")
            }
        };

        Costume {
            costume_asset: self,
            skin
        }
    }
}

#[derive(Debug)]
pub struct Costume {
    pub costume_asset: CostumeAsset,
    pub skin: Rc<dyn Skin>
}
