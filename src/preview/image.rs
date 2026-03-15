#[cfg(feature = "image-preview")]
use std::path::Path;

#[cfg(feature = "image-preview")]
use image::ImageReader;
#[cfg(feature = "image-preview")]
use ratatui_image::picker::Picker;
#[cfg(feature = "image-preview")]
use ratatui_image::protocol::StatefulProtocol;

/// Decode an image file and create a `StatefulProtocol` for rendering.
#[cfg(feature = "image-preview")]
pub fn load_image(path: &Path, picker: &Picker) -> Result<StatefulProtocol, String> {
    let img = ImageReader::open(path)
        .map_err(|e| format!("Failed to open image: {e}"))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {e}"))?;
    Ok(picker.new_resize_protocol(img))
}

#[cfg(test)]
#[cfg(feature = "image-preview")]
mod tests {
    use super::*;

    #[test]
    fn load_image_fails_on_non_image_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("not_an_image.txt");
        std::fs::write(&path, "hello world").unwrap();

        let picker = Picker::halfblocks();
        let result = load_image(&path, &picker);
        assert!(result.is_err());
    }

    #[test]
    fn load_image_succeeds_on_valid_png() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.png");

        // Create a minimal 1x1 red PNG
        let img = image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]));
        img.save(&path).unwrap();

        let picker = Picker::halfblocks();
        let result = load_image(&path, &picker);
        assert!(result.is_ok());
    }
}
