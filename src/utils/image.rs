use image_convert::{to_jpg, ImageResource, JPGConfig};
use std::fs::rename;
use std::path::{Path, PathBuf};

pub fn convert(
  input_path: &Path,
  (width, height): (u16, u16),
  shrink_only: bool,
) -> Result<(), String> {
  let mut config = JPGConfig::new();
  config.width = width;
  config.height = height;
  config.shrink_only = shrink_only;

  // convert
  let input = ImageResource::from_path(input_path);
  let output_path = &get_output_path(&input_path)?;
  let mut output = ImageResource::from_path(output_path);
  to_jpg(&mut output, &input, &config).map_err(|e| e.to_string())?;

  // remove temp
  rename(output_path, input_path).map_err(|e| e.to_string())?;

  Ok(())
}

fn get_output_path(input_path: &Path) -> Result<PathBuf, String> {
  let input_fname = input_path
    .file_name()
    .ok_or("No image filename")?
    .to_str()
    .ok_or("Can't convert os string to string")?;
  let output_fname = input_fname.to_string() + ".out.jpg";
  let output_path = std::env::temp_dir().join(Path::new(&output_fname));

  Ok(output_path)
}
