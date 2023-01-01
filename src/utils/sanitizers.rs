use sanitize_html::rules::predefined::DEFAULT;
use sanitize_html::sanitize_str;

pub fn html_sanity(s: &str) -> Result<String, String> {
  Ok(sanitize_str(&DEFAULT, s).map_err(|e| format!("HTML_SANITY: \"{}\"", e))?)
}

pub fn psql_sanity(s: &str) -> String {
  s.replace("'", "''")
}
