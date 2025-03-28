use kindle_screensaver::{convert_to_kindle, ConversionOptions, KindleModel, DitheringAlgorithm, ResizingMethod};
use std::path::Path;

fn main() {
    let input_path = Path::new("80.jpg");
    
    let options = ConversionOptions {
        model: KindleModel::Paperwhite,
        dithering: DitheringAlgorithm::FloydSteinberg,
        optimize_contrast: true,
        resizing_method: ResizingMethod::Lanczos3,
    };
    
    println!("Converting image for Kindle Paperwhite...");
    match convert_to_kindle(input_path, options, None) {
        Ok(output_path) => println!("Successfully converted to: {}", output_path),
        Err(e) => eprintln!("Error: {}", e),
    }
    
    let colorsoft_options = ConversionOptions {
        model: KindleModel::Colorsoft,
        dithering: DitheringAlgorithm::FloydSteinberg,
        optimize_contrast: true,
        resizing_method: ResizingMethod::Lanczos3,
    };
    
    println!("Converting image for Kindle Colorsoft...");
    match convert_to_kindle(input_path, colorsoft_options, Some(Path::new("kindle_colorsoft.png"))) {
        Ok(output_path) => println!("Successfully converted to: {}", output_path),
        Err(e) => eprintln!("Error: {}", e),
    }
}

