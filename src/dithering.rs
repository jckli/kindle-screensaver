use image::{DynamicImage, ImageBuffer, Rgba};
use rayon::prelude::*;
use rayon::iter::ParallelBridge;

pub fn apply_threshold(value: u8, threshold: u8, levels: usize) -> u8 {
    if levels <= 1 { return value; }
    
    if value > threshold {
        ((value as usize * levels / 256) * 255 / (levels - 1)) as u8
    } else {
        let level = value as usize * levels / 256;
        if level == 0 {
            0
        } else {
            ((level - 1) * 255 / (levels - 1)) as u8
        }
    }
}

/// apply Floyd-Steinberg dithering to a grayscale image
pub fn apply_floyd_steinberg_dithering(img: DynamicImage) -> DynamicImage {
    let grayscale = img.to_rgba8();
    let (width, height) = grayscale.dimensions();
    
    let mut dithered = grayscale.clone();
    
    let levels = 16;
    if levels <= 1 {
        return DynamicImage::ImageRgba8(dithered);
    }
    
    let step = 256_usize / levels;
    
    for y in 0..height-1 {
        for x in 1..width-1 {
            let px = dithered.get_pixel(x, y);
            let old_val = px[0];
            
            let new_val = (old_val as usize / step * step) as u8;
            
            let new_px = [new_val, new_val, new_val, px[3]];
            dithered.put_pixel(x, y, Rgba(new_px));
            
            let error = old_val as i16 - new_val as i16;
            
            if x < width-1 {
                let mut px = dithered.get_pixel(x+1, y).0;
                let val = (px[0] as i16 + error * 7 / 16).clamp(0, 255) as u8;
                px[0] = val;
                px[1] = val;
                px[2] = val;
                dithered.put_pixel(x+1, y, Rgba(px));
            }
            
            if x > 0 && y < height-1 {
                let mut px = dithered.get_pixel(x-1, y+1).0;
                let val = (px[0] as i16 + error * 3 / 16).clamp(0, 255) as u8;
                px[0] = val;
                px[1] = val;
                px[2] = val;
                dithered.put_pixel(x-1, y+1, Rgba(px));
            }
            
            if y < height-1 {
                let mut px = dithered.get_pixel(x, y+1).0;
                let val = (px[0] as i16 + error * 5 / 16).clamp(0, 255) as u8;
                px[0] = val;
                px[1] = val;
                px[2] = val;
                dithered.put_pixel(x, y+1, Rgba(px));
            }
            
            if x < width-1 && y < height-1 {
                let mut px = dithered.get_pixel(x+1, y+1).0;
                let val = (px[0] as i16 + error * 1 / 16).clamp(0, 255) as u8;
                px[0] = val;
                px[1] = val;
                px[2] = val;
                dithered.put_pixel(x+1, y+1, Rgba(px));
            }
        }
    }
    
    DynamicImage::ImageRgba8(dithered)
}

/// apply ordered dithering to a grayscale image
pub fn apply_ordered_dithering(img: DynamicImage) -> DynamicImage {
    let grayscale = img.to_rgba8();
    let (width, height) = grayscale.dimensions();
    
    let mut dithered = ImageBuffer::new(width, height);
    
    let bayer = [
        [0, 8, 2, 10],
        [12, 4, 14, 6],
        [3, 11, 1, 9],
        [15, 7, 13, 5]
    ];
    
    let bayer_scale = 1.0 / 16.0 * 255.0;
    
    let levels = 16;
    
    dithered.enumerate_pixels_mut().par_bridge().for_each(|(x, y, pixel)| {
        let source_pixel = grayscale.get_pixel(x, y);
        let gray_val = source_pixel[0];
        
        let threshold = (bayer[y as usize % 4][x as usize % 4] as f32 * bayer_scale) as u8;
        
        let new_val = if gray_val > threshold {
            ((gray_val as usize * levels / 256) * 255 / (levels - 1)) as u8
        } else {
            let level = gray_val as usize * levels / 256;
            if level == 0 {
                0
            } else {
                ((level - 1) * 255 / (levels - 1)) as u8
            }
        };
        
        *pixel = Rgba([new_val, new_val, new_val, source_pixel[3]]);
    });
    
    DynamicImage::ImageRgba8(dithered)
}

/// apply color dithering (Floyd-Steinberg) for color Kindles
pub fn apply_color_dithering(img: DynamicImage) -> DynamicImage {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    
    let mut dithered = rgba.clone();
    
    let levels = 6;
    if levels <= 1 {
        return DynamicImage::ImageRgba8(dithered);
    }
    
    let step = 256_usize / levels;
    
    for y in 0..height-1 {
        for x in 1..width-1 {
            let old_px = dithered.get_pixel(x, y).0;
            
            let new_r = (old_px[0] as usize / step * step) as u8;
            let new_g = (old_px[1] as usize / step * step) as u8;
            let new_b = (old_px[2] as usize / step * step) as u8;
            
            let new_px = [new_r, new_g, new_b, old_px[3]];
            dithered.put_pixel(x, y, Rgba(new_px));
            
            let r_error = old_px[0] as i16 - new_r as i16;
            let g_error = old_px[1] as i16 - new_g as i16;
            let b_error = old_px[2] as i16 - new_b as i16;
            
            if x < width-1 {
                let mut px = dithered.get_pixel(x+1, y).0;
                px[0] = (px[0] as i16 + r_error * 7 / 16).clamp(0, 255) as u8;
                px[1] = (px[1] as i16 + g_error * 7 / 16).clamp(0, 255) as u8;
                px[2] = (px[2] as i16 + b_error * 7 / 16).clamp(0, 255) as u8;
                dithered.put_pixel(x+1, y, Rgba(px));
            }
            
            if x > 0 && y < height-1 {
                let mut px = dithered.get_pixel(x-1, y+1).0;
                px[0] = (px[0] as i16 + r_error * 3 / 16).clamp(0, 255) as u8;
                px[1] = (px[1] as i16 + g_error * 3 / 16).clamp(0, 255) as u8;
                px[2] = (px[2] as i16 + b_error * 3 / 16).clamp(0, 255) as u8;
                dithered.put_pixel(x-1, y+1, Rgba(px));
            }
            
            if y < height-1 {
                let mut px = dithered.get_pixel(x, y+1).0;
                px[0] = (px[0] as i16 + r_error * 5 / 16).clamp(0, 255) as u8;
                px[1] = (px[1] as i16 + g_error * 5 / 16).clamp(0, 255) as u8;
                px[2] = (px[2] as i16 + b_error * 5 / 16).clamp(0, 255) as u8;
                dithered.put_pixel(x, y+1, Rgba(px));
            }
            
            if x < width-1 && y < height-1 {
                let mut px = dithered.get_pixel(x+1, y+1).0;
                px[0] = (px[0] as i16 + r_error * 1 / 16).clamp(0, 255) as u8;
                px[1] = (px[1] as i16 + g_error * 1 / 16).clamp(0, 255) as u8;
                px[2] = (px[2] as i16 + b_error * 1 / 16).clamp(0, 255) as u8;
                dithered.put_pixel(x+1, y+1, Rgba(px));
            }
        }
    }
    
    DynamicImage::ImageRgba8(dithered)
}

/// apply color ordered dithering for color Kindles
pub fn apply_color_ordered_dithering(img: DynamicImage) -> DynamicImage {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    
    let mut dithered = ImageBuffer::new(width, height);
    
    let bayer = [
        [0, 8, 2, 10],
        [12, 4, 14, 6],
        [3, 11, 1, 9],
        [15, 7, 13, 5]
    ];
    
    let bayer_scale = 1.0 / 16.0 * 255.0;
    
    let levels = 6;
    
    dithered.enumerate_pixels_mut().par_bridge().for_each(|(x, y, pixel)| {
        let source_pixel = rgba.get_pixel(x, y);
        
        let threshold = (bayer[y as usize % 4][x as usize % 4] as f32 * bayer_scale) as u8;
        
        let r = apply_threshold(source_pixel[0], threshold, levels);
        let g = apply_threshold(source_pixel[1], threshold, levels);
        let b = apply_threshold(source_pixel[2], threshold, levels);
        
        *pixel = Rgba([r, g, b, source_pixel[3]]);
    });
    
    DynamicImage::ImageRgba8(dithered)
}

