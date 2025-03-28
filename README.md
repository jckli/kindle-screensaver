# kindle-screensaver

A library for converting images to Kindle-compatible screensavers

## Usage

You can specify different conversion options.

> **Note:** Some conversion options may take a long time.

```rust
let input_path = Path::new("80.jpg");

let options = ConversionOptions {
    model: KindleModel::Paperwhite,
    dithering: DitheringAlgorithm::FloydSteinberg,
    optimize_contrast: true,
    resizing_method: ResizingMethod::Lanczos3,
};

match convert_to_kindle(input_path, options, None) {
    Ok(output_path) => println!("Successfully converted to: {}", output_path),
    Err(e) => eprintln!("Error: {}", e),
}


```
