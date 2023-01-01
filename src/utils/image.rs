// Conversion
use image_convert::{to_jpg, ImageResource, JPGConfig};
use std::fs::rename;
use std::path::{Path, PathBuf};

// Image processing
extern crate image;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use rocket::tokio::io::AsyncReadExt;

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

pub struct ImgData {
  pub width: i32,
  pub height: i32,
  pub buf: Vec<u8>,
}

pub async fn read_image(filename: &PathBuf) -> Result<ImgData, String> {
  let mut fh = rocket::tokio::fs::File::open(filename)
    .await
    .map_err(|e| format!("{e:?}"))?;
  let mut buf = Vec::new();
  fh.read_to_end(&mut buf)
    .await
    .map_err(|e| format!("{e:?}"))?;
  let image = ImageReader::new(std::io::Cursor::new(&buf))
    .with_guessed_format()
    .map_err(|e| format!("{e:?}"))?
    .decode()
    .map_err(|e| format!("{e:?}"))?;
  let width = i32::try_from(image.width()).map_err(|e| format!("{e:?}"))?;
  let height = i32::try_from(image.height()).map_err(|e| format!("{e:?}"))?;
  drop(fh);
  return Ok(ImgData { width, height, buf });
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
