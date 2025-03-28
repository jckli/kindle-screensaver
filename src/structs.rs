use thiserror::Error;
use image::imageops;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KindleModel {
    Basic,
    Paperwhite,
    Colorsoft,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DitheringAlgorithm {
    None,
    FloydSteinberg,
    Ordered,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizingMethod {
    Nearest,
    Triangle,
    Lanczos3,
}


impl ResizingMethod {
    pub fn to_filter_type(&self) -> imageops::FilterType {
        match self {
            ResizingMethod::Nearest => imageops::FilterType::Nearest,
            ResizingMethod::Triangle => imageops::FilterType::Triangle, 
            ResizingMethod::Lanczos3 => imageops::FilterType::Lanczos3,
        }
    }
}

impl ConversionOptions {
    pub fn filter_type(&self) -> imageops::FilterType {
        self.resizing_method.to_filter_type()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ConversionOptions {
    pub model: KindleModel,
    pub dithering: DitheringAlgorithm,
    pub optimize_contrast: bool,
    pub resizing_method: ResizingMethod,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            model: KindleModel::Paperwhite,
            dithering: DitheringAlgorithm::None,
            optimize_contrast: true,
            resizing_method: ResizingMethod::Nearest,
        }
    }
}

impl KindleModel {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            KindleModel::Basic => (1072, 1448),
            KindleModel::Paperwhite => (1072, 1448),
            KindleModel::Colorsoft => (1272, 1696),
        }
    }

    pub fn bit_depth(&self) -> u8 {
        match self {
            KindleModel::Basic => 8,
            KindleModel::Paperwhite => 8,
            KindleModel::Colorsoft => 32,
        }
    }
    
    pub fn is_color(&self) -> bool {
        // only kindle colorsoft has color currently
        match self {
            KindleModel::Colorsoft => true,
            _ => false,
        }
    }
}

#[derive(Error, Debug)]
pub enum KindleError {
    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Unsupported image format")]
    UnsupportedFormat,
    
    #[error("Invalid input data")]
    InvalidData,
}
