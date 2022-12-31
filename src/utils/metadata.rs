pub struct MetaData {
  pub creation_time: f64,
  pub camera_make: String,
  pub camera_model: String,
  pub orientation: i32,
  pub horizontal_ppi: i32,
  pub vertical_ppi: i32,
  pub shutter_speed: f64,
  pub color_space: String,
}

impl MetaData {
  pub fn parse(xml: impl Into<String>) -> Result<MetaData, String> {
    let errf = |e: minidom::error::Error| e.to_string();
    let root: minidom::Element = xml.into().parse().map_err(errf)?;

    Ok(MetaData {
      creation_time:  Self::get_el::<f64>(&root, "creationTime").unwrap_or(0.0),
      camera_make:    Self::get_el::<String>(&root, "cameraMake")?,
      camera_model:   Self::get_el::<String>(&root, "cameraModel")?,
      orientation:    Self::get_el::<i32>(&root, "orientation").unwrap_or(0),
      horizontal_ppi: Self::get_el::<i32>(&root, "horizontalPpi").unwrap_or(0),
      vertical_ppi:   Self::get_el::<i32>(&root, "verticalPpi").unwrap_or(0),
      shutter_speed:  Self::get_el::<f64>(&root, "shutterSpeed").unwrap_or(0.0),
      color_space:    Self::get_el::<String>(&root, "colorSpace")?,
    })
  }

  fn get_el<T: std::str::FromStr>(root: &minidom::Element, child: &str) -> Result<T, String> {
    match root.get_child(child, root.ns().as_str()) {
      Some(el) => el.text().parse::<T>()
        .or(Err(format!("XML: failed to parse field \"{}\"", child))),
      None => Err(format!("XML: No element named \"{}\"", child)),
    }
  }
}
