use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui;
use strum::IntoEnumIterator;

use crate::puzzle::Orientation;


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, strum::EnumIter)]
pub enum AssetType {
    Straight,
    StraightPowered,
    Corner,
    CornerPowered,
    TIntersection,
    TIntersectionPowered,
    CrossIntersection,
    CrossIntersectionPowered,
    DeadEnd,
    DeadEndPowered,
    Wall,
    Source,
    SourcePowered,
    Drain,
    DrainPowered,
}

fn filename_from_asset_type(asset_type: AssetType) -> &'static str {
    let assets = HashMap::from([
        (AssetType::Straight, "straight.png"),
        (AssetType::StraightPowered, "straight_powered.png"),
        (AssetType::Corner, "corner.png"),
        (AssetType::CornerPowered, "corner_powered.png"),
        (AssetType::TIntersection, "t_intersection.png"),
        (
            AssetType::TIntersectionPowered,
            "t_intersection_powered.png",
        ),
        (AssetType::CrossIntersection, "cross_intersection.png"),
        (
            AssetType::CrossIntersectionPowered,
            "cross_intersection_powered.png",
        ),
        (AssetType::DeadEnd, "dead_end.png"),
        (AssetType::DeadEndPowered, "dead_end_powered.png"),
        (AssetType::Wall, "wall.png"),
        (AssetType::Source, "source.png"),
        (AssetType::SourcePowered, "source_powered.png"),
        (AssetType::Drain, "drain.png"),
        (AssetType::DrainPowered, "drain_powered.png"),
    ]);
    assets.get(&asset_type).expect("path to asset not found")
}

#[derive(Clone, Default)]
pub struct Assets {
    assets: HashMap<AssetType, Vec<egui::TextureHandle>>,
}

impl Assets {
    /// Create a new empty asset collection.
    pub fn new() -> Self {
        Assets {
            assets: HashMap::new(),
        }
    }

    /// Load all assets from hardcoded paths.
    pub fn load_all(&mut self, context: &egui::Context) {
        for asset_type in AssetType::iter() {
            self.load(asset_type, context);
        }
    }

    /// Load a specific asset type.
    #[doc(hidden)]
    fn load(&mut self, asset_type: AssetType, context: &egui::Context) {
        let root: PathBuf = PathBuf::from("assets/40");
        let path = root.join(filename_from_asset_type(asset_type));

        let image = image::ImageReader::open(path)
            .expect("could not load image")
            .decode()
            .expect("could not decode image");

        let size = [image.width() as _, image.height() as _];

        let assets = Orientation::iter()
            .map(|rotation| {
                let image = match rotation {
                    Orientation::Basic => &image,
                    Orientation::Ccw90 => &image.rotate270(),
                    Orientation::Ccw180 => &image.rotate180(),
                    Orientation::Ccw270 => &image.rotate90(),
                };
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                context.load_texture("texture", color_image, Default::default())
            })
            .collect();
        self.assets.insert(asset_type, assets);
    }

    pub fn get_rotated(&self, asset_type: AssetType, rotation: Orientation) -> Option<egui::TextureHandle> {
        self.assets
            .get(&asset_type)
            .and_then(|handles| handles.get(rotation as usize))
            .cloned()
    }
}
