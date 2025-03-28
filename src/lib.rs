pub mod structs;
pub mod dithering;

use bytes::Bytes;
use rayon::prelude::*;
use rayon::iter::ParallelBridge;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgba};
use std::io::Cursor;
use std::path::Path;
use crate::dithering::{apply_floyd_steinberg_dithering, apply_ordered_dithering, apply_color_dithering, apply_color_ordered_dithering};

pub use structs::{KindleModel, DitheringAlgorithm, ConversionOptions, KindleError, ResizingMethod};

pub type Result<T> = std::result::Result<T, KindleError>;

pub fn convert_to_kindle(
    input_path: &Path,
    options: ConversionOptions,
    output_path: Option<&Path>,
) -> Result<String> {
    let img = image::open(input_path)?;
    
    let kindle_img = process_for_kindle(img, &options)?;
    
    let output = match output_path {
        Some(path) => path.to_path_buf(),
        None => {
            let stem = input_path.file_stem().unwrap_or_default();
            let extension = "png";
            let filename = format!("{}_kindle.{}", stem.to_string_lossy(), extension);
            input_path.with_file_name(filename)
        }
    };
    
    kindle_img.save(&output)?;
    
    Ok(output.to_string_lossy().to_string())
}

pub fn convert_from_bytes(
    input_bytes: &[u8], 
    options: ConversionOptions
) -> Result<Bytes> {
    // load image from bytes
    let img = image::load_from_memory(input_bytes)?;
    
    // process image
    let kindle_img = process_for_kindle(img, &options)?;
    
    // convert to bytes
    let mut buffer = Cursor::new(Vec::new());
    kindle_img.write_to(&mut buffer, ImageFormat::Png)?;
    
    Ok(Bytes::from(buffer.into_inner()))
}

fn process_for_kindle(img: DynamicImage, options: &ConversionOptions) -> Result<DynamicImage> {
    
    if !options.optimize_contrast && options.dithering == DitheringAlgorithm::None {
        let resized = resize_maintain_aspect_ratio(&img, options);
        if !options.model.is_color() {
            return Ok(convert_to_grayscale(resized));
        } else {
            return Ok(resized);
        }
    }
    
    let processed = resize_maintain_aspect_ratio(&img, options);
    
    let processed = if options.optimize_contrast {
        optimize_contrast_for_eink(processed)
    } else {
        processed
    };
    
    if !options.model.is_color() {
        // convert to grayscale
        let grayscale = convert_to_grayscale(processed);
            
        match options.dithering {
            DitheringAlgorithm::None => Ok(grayscale),
            DitheringAlgorithm::FloydSteinberg => Ok(apply_floyd_steinberg_dithering(grayscale)),
            DitheringAlgorithm::Ordered => Ok(apply_ordered_dithering(grayscale)),
        }
    } else {
        match options.dithering {
            DitheringAlgorithm::None => Ok(processed),
            DitheringAlgorithm::FloydSteinberg => Ok(apply_color_dithering(processed)),
            DitheringAlgorithm::Ordered => Ok(apply_color_ordered_dithering(processed)),
        }
    }
}

pub fn batch_convert(
    input_path: &Path,
    options_list: &[ConversionOptions],
    output_dir: &Path,
) -> Result<Vec<String>> {
    let img = image::open(input_path)?;
    
    let stem = input_path.file_stem().unwrap_or_default();
    
    options_list.par_iter()
        .map(|options| {
            let kindle_img = process_for_kindle(img.clone(), options)?;
            
            let model_name = format!("{:?}", options.model).to_lowercase();
            let filename = format!("{}_{}.png", stem.to_string_lossy(), model_name);
            let output_path = output_dir.join(filename);
            
            kindle_img.save(&output_path)?;
            Ok(output_path.to_string_lossy().to_string())
        })
        .collect()
}

fn resize_maintain_aspect_ratio(img: &DynamicImage, options: &ConversionOptions) -> DynamicImage {
    let (target_width, target_height) = options.model.dimensions();
    let (width, height) = img.dimensions();
    
    let width_scale = target_width as f32 / width as f32;
    let height_scale = target_height as f32 / height as f32;
    
    let scale = width_scale.max(height_scale);
    
    let scaled_width = (width as f32 * scale) as u32;
    let scaled_height = (height as f32 * scale) as u32;

    let mut resized = img.resize(scaled_width, scaled_height, options.filter_type());
    
    let x_offset = (scaled_width - target_width) / 2;
    let y_offset = (scaled_height - target_height) / 2;
    
    resized.crop(x_offset, y_offset, target_width, target_height)
}

fn convert_to_grayscale(img: DynamicImage) -> DynamicImage {
    img.grayscale()
}

fn optimize_contrast_for_eink(img: DynamicImage) -> DynamicImage {
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    
    let pixel_values: Vec<_> = rgba_img.pixels().collect();
    
    let (min_val, max_val) = pixel_values.par_iter()
        .map(|pixel| {
            let luma = (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) as u8;
            (luma, luma)
        })
        .reduce(
            || (255, 0),
            |(min1, max1), (min2, max2)| (min1.min(min2), max1.max(max2))
        );
    
    let range = if max_val > min_val { max_val - min_val } else { 1 };
    
    let lut: Vec<u8> = (0..=255)
        .map(|v| {
            let scaled = (v as f32 - min_val as f32) / range as f32 * 255.0;
            scaled.clamp(0.0, 255.0) as u8
        })
        .collect();
    
    let mut buffer = ImageBuffer::new(width, height);
    
    buffer.enumerate_pixels_mut().par_bridge().for_each(|(x, y, pixel)| {
        let source_pixel = rgba_img.get_pixel(x, y);
        
        let r = lut[source_pixel[0] as usize];
        let g = lut[source_pixel[1] as usize];
        let b = lut[source_pixel[2] as usize];
        
        *pixel = Rgba([r, g, b, source_pixel[3]]);
    });
    
    DynamicImage::ImageRgba8(buffer)
}
