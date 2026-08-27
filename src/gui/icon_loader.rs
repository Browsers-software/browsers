use std::collections::HashMap;
use std::path::Path;

use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use tracing::debug;

// decodes icon files into slint::Images, with a small cache since the same browser icon shows up
// across profiles and gets asked for again on every refresh
#[derive(Default)]
pub struct IconLoader {
    cache: HashMap<String, Image>,
}

impl IconLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, icon_path: &str) -> Image {
        if icon_path.is_empty() {
            return Image::default();
        }

        if let Some(image) = self.cache.get(icon_path) {
            return image.clone();
        }

        let image = decode_icon(icon_path).unwrap_or_default();
        self.cache.insert(icon_path.to_string(), image.clone());
        image
    }
}

fn decode_icon(icon_path: &str) -> Option<Image> {
    let path = Path::new(icon_path);
    let dynamic_image = match image::ImageReader::open(path).ok()?.decode() {
        Ok(img) => img,
        Err(error) => {
            debug!("failed to decode icon {}: {:?}", icon_path, error);
            return None;
        }
    };

    let rgba = dynamic_image.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(rgba.as_raw(), width, height);
    Some(Image::from_rgba8(buffer))
}
