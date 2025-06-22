use crate::error::Result;
use candle_core::{DType, Device, Tensor};
use image::DynamicImage;

pub fn load_image(path: impl AsRef<std::path::Path>) -> Result<DynamicImage> {
    Ok(image::ImageReader::open(&path)?.decode()?)
}

pub fn image_to_tensor(
    path: impl AsRef<std::path::Path>,
    resize_shape: Option<(u32, u32)>,
) -> Result<Tensor> {
    let original_img = load_image(&path)?;
    let img = match resize_shape {
        Some((width, height)) => {
            original_img.resize_to_fill(width, height, image::imageops::FilterType::Triangle)
        }
        None => original_img,
    };
    let width = img.width() as usize;
    let height = img.height() as usize;
    let raw_data = img.into_rgb8().into_raw();
    let data = Tensor::from_vec(raw_data, (width, height, 3), &Device::Cpu)?.permute((2, 0, 1))?;
    Ok((data.to_dtype(DType::F32)? / 255.0)?)
}
