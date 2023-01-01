use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

pub struct CSP {}

impl CSP {
  const HEADERS: &'static str = r#"default-src 'none'; img-src 'self'; style-src 'self' https://stackpath.bootstrapcdn.com; script-src 'self' https://stackpath.bootstrapcdn.com https://cdnjs.cloudflare.com https://code.jquery.com; object-src 'none'; frame-ancestors 'none'; form-action 'self';"#;
}

#[rocket::async_trait]
impl Fairing for CSP {
  fn info(&self) -> Info {
    Info {
      name: "CSP headers",
      kind: Kind::Response,
    }
  }

  async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
    response.set_header(Header::new("Content-Security-Policy", Self::HEADERS));
  }
}
