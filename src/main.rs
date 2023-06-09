// Rocket
#[macro_use]
extern crate rocket;
use rocket::figment::{
  util::map,
  value::{Map, Value},
};
use rocket::{
  fairing::AdHoc,
  fs::{FileServer, TempFile},
  http::ContentType,
  response::Redirect,
  serde::{Deserialize, Serialize},
  shield, Build, Rocket,
};
use rocket_csrf::form::CsrfForm;
use rocket_csrf::CsrfToken;
use rocket_dyn_templates::{context, Template};
use rocket_sync_db_pools::{database, postgres};

// My
#[macro_use]
mod utils;
use crate::utils::{
  fairings::CSP,
  image::{convert as image_convert, read_image},
  metadata::MetaData,
  sanitizers::{html_sanity, psql_sanity},
};

// Misc
use http::status::StatusCode;
use rand::prelude::*;

////////////////////
// Macro definitions
////////////////////

define_http!($ error_template templ);
fn error_template(code: u16) -> Template {
  let scode = StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST);
  Template::render(
    "error",
    context! {
      reason: scode.canonical_reason().unwrap_or(""),
      code: code
    },
  )
}

define_http!($ error_redirect redir);
fn error_redirect(code: u16) -> Redirect {
  Redirect::to(format!("/error/{}", code))
}

////////////////////
// Data
////////////////////

#[database("postgres")]
struct MyPgDatabase(postgres::Client);

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ImagesList {
  images: Vec<ImageListItem>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ImageListItem {
  id: String,
  path: String,
  width: i32,
  height: i32,
  title: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ImageShow {
  title: String,
  path: String,
  id: String,
  comments: Vec<ImageComment>,
  metadata: Option<ImageMetadata>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ImageResult {
  id: String,
  title: String,
  path: String,
  width: i32,
  height: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct SearchResult {
  query: String,
  results: Vec<ImageResult>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ImageComment {
  text: String,
  user: String,
}

#[derive(FromForm)]
struct PostComment {
  comment: String,
  user_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ImageMetadata {
  creation_time: f64,
  camera_make: String,
  camera_model: String,
  orientation: i32,
  horizontal_ppi: i32,
  vertical_ppi: i32,
  shutter_speed: f64,
  color_space: String,
}

#[derive(FromForm)]
struct UploadImage<'v> {
  title: String,
  private: bool,
  file: TempFile<'v>,
  metadata: Option<TempFile<'v>>,
}

////////////////////
// API
////////////////////

#[post("/image/<imageid>/comments/post", data = "<comment>")]
async fn comment(conn: MyPgDatabase, imageid: String, comment: CsrfForm<PostComment>) -> Redirect {
  let user_name = redir!(res html_sanity(&comment.user_name));
  let comment = redir!(res html_sanity(&comment.comment));
  let image_id = imageid.clone();
  redir!(res conn.run(move |conn| {
        conn.query("INSERT INTO comments (image_id, user_name, comment) VALUES ($1, $2, $3)", 
            &[&image_id, &user_name, &comment])
    }).await);
  Redirect::to(format!("/image/{}", imageid))
}

async fn img2jpg(
  form: &mut CsrfForm<UploadImage<'_>>,
) -> Result<impl FnOnce(&mut postgres::Transaction) -> Result<String, String>, Redirect> {
  let imgpath = std::env::temp_dir().join(redir!(opt form.file.name() => Err));
  redir!(res form.file.persist_to(&imgpath).await => Err);
  redir!(res image_convert(imgpath.as_path(), (2048, 2048), true) => Err);
  let img = redir!(res read_image(&imgpath).await => Err);
  redir!(res rocket::tokio::fs::remove_file(&imgpath).await => Err);

  let title = form.title.clone();
  let private = form.private.clone();
  let path = format!(
    "{}{}",
    String::from(redir!(opt form.file.name() => Err)),
    suffix()
  );

  return Ok(move |transaction: &mut postgres::Transaction| {
    let res = transaction.query(
            "INSERT INTO images (title, path, width, height, private, content) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id", 
            &[&title, &path, &img.width, &img.height, &private, &img.buf]).map_err(|e| e.to_string())?;
    return Ok(res.get(0).ok_or("Can't get id while inserting!")?.get(0));
  });

  fn suffix() -> String {
    let mut rng = rand::thread_rng();
    rng.gen::<u16>().to_string()
  }
}

async fn metadata(
  form: &mut CsrfForm<UploadImage<'_>>,
) -> Result<Option<impl FnOnce(&mut postgres::Transaction, &String) -> Result<(), String>>, Redirect>
{
  if let Some(metadata) = &mut form.metadata {
    let name_result = metadata.name();
    if name_result.is_some() {
      let name = redir!(opt name_result => Err);
      let path = format!("{}", std::env::temp_dir().join(name).display());
      redir!(res metadata.persist_to(&path).await => Err);

      let md =
        redir!(res MetaData::parse(redir!(res std::fs::read_to_string(&path) => Err)) => Err);

      redir!(res rocket::tokio::fs::remove_file(&path).await => Err);
      return Ok(Some(
        move |transaction: &mut postgres::Transaction, id: &String| {
          transaction.query(
                    "INSERT INTO metadata (image_id, creationtime, camera_make, camera_model, orientation, horizontal_ppi, vertical_ppi, shutter_speed, color_space) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)", 
                    &[&id, &md.creation_time, &md.camera_make, &md.camera_model, &md.orientation, &md.horizontal_ppi, &md.vertical_ppi, &md.shutter_speed, &md.color_space]
                ).map_err(|e| e.to_string())?;
          Ok(())
        },
      ));
    }
  }

  Ok(None)
}

#[post("/upload", data = "<form>")]
async fn upload_post(conn: MyPgDatabase, mut form: CsrfForm<UploadImage<'_>>) -> Redirect {
  let imgfn = match img2jpg(&mut form).await {
    Ok(v) => v,
    Err(r) => return r,
  };
  let metadatafn = match metadata(&mut form).await {
    Ok(v) => v,
    Err(r) => return r,
  };

  // Put it into atomic transaction
  conn
    .run(move |conn| -> Redirect {
      let mut t = null!(res conn.transaction(), Redirect::to("/error/500"));
      let id = null!(res imgfn(&mut t), { t.rollback().unwrap_or(()); Redirect::to("/error/500") });
      if let Some(mdfn) = metadatafn {
        null!(res mdfn(&mut t, &id), { t.rollback().unwrap_or(()); Redirect::to("/error/500") });
      }

      null!(res t.commit(), Redirect::to("/error/500"));
      Redirect::to(format!("/image/{}", id))
    })
    .await
}

#[get("/upload", rank = 2)]
async fn upload_csrf() -> Redirect {
  Redirect::to("/upload")
}

#[get("/upload", rank = 1)]
async fn upload(csrf_token: CsrfToken) -> Template {
  let csrf_token = csrf_token.authenticity_token();
  Template::render("upload", context! { csrf_token })
}

#[get("/search?<q>")]
async fn search(conn: MyPgDatabase, q: String) -> Template {
  let mut results: Vec<ImageResult> = Vec::new();
  let qs = format!("SELECT id, path, width, height, title FROM images WHERE NOT private AND (to_tsvector('simple', title) @@ plainto_tsquery('simple', '{}'))", psql_sanity(&q.as_str()));
  let r = templ!(res conn.run(move |conn| {
        conn.query(&qs, &[])
    }).await);
  for row in r.iter() {
    results.push(ImageResult {
      id: row.get(0),
      path: row.get(1),
      width: row.get(2),
      height: row.get(3),
      title: row.get(4),
    });
  }
  let result = SearchResult {
    query: q,
    results: results,
  };
  Template::render("search", result)
}

#[get("/image/<imageid>", rank = 2)]
async fn images_name_csrf(imageid: String) -> Redirect {
  Redirect::to(format!("/image/{}", imageid))
}

#[get("/image/<imageid>", rank = 1)]
async fn images_name(csrf_token: CsrfToken, conn: MyPgDatabase, imageid: String) -> Template {
  let image_id = imageid.clone();
  let r = templ!(res conn.run(move |conn| {
        conn.query_one("SELECT path, title, id FROM images WHERE id = $1", &[&image_id])
    }).await);
  let path: String = r.get(0);
  let title: String = r.get(1);
  let id: String = r.get(2);

  let image_id = imageid.clone();
  let metadata_result = conn.run(move |conn| {
        conn.query_one("SELECT creationtime, camera_make, camera_model, orientation, horizontal_ppi, vertical_ppi, shutter_speed, color_space FROM metadata WHERE image_id = $1", &[&image_id])
    }).await;

  let comment_result = templ!(res conn.run(move |conn| {
        conn.query("SELECT user_name, comment from comments where image_id = $1", &[&imageid])
    }).await);
  let mut comments: Vec<ImageComment> = Vec::new();
  for comment in comment_result.iter() {
    comments.push(ImageComment {
      user: comment.get(0),
      text: comment.get(1),
    })
  }
  let metadata = match metadata_result {
    Ok(row) => Some(ImageMetadata {
      creation_time: row.get(0),
      camera_make: row.get(1),
      camera_model: row.get(2),
      orientation: row.get(3),
      horizontal_ppi: row.get(4),
      vertical_ppi: row.get(5),
      shutter_speed: row.get(6),
      color_space: row.get(7),
    }),
    Err(_) => None,
  };
  let csrf_token = csrf_token.authenticity_token();
  Template::render(
    "image",
    context! { csrf_token, title, path, id, comments, metadata },
  )
}

#[get("/img/<name>")]
async fn img(conn: MyPgDatabase, name: String) -> Result<(ContentType, Vec<u8>), Template> {
  let r = conn
    .run(move |conn| {
      conn.query_one(
        "SELECT content from images where path = $1 LIMIT 1",
        &[&name],
      )
    })
    .await
    .map_err(|_| error_template(204))?;
  let content = r.get(0);
  return Ok((ContentType::JPEG, content));
}

#[get("/error/<code>")]
async fn error(code: u16) -> Template {
  return error_template(code);
}

#[get("/")]
async fn images(conn: MyPgDatabase) -> Template {
  let r = conn
    .run(|conn| {
      conn.query(
        "SELECT id, path, width, height, title FROM images WHERE NOT private",
        &[],
      )
    })
    .await;

  match r {
    Ok(l) => {
      let image_list = l
        .iter()
        .map(|row| -> ImageListItem {
          ImageListItem {
            id: row.get(0),
            path: row.get(1),
            width: row.get(2),
            height: row.get(3),
            title: row.get(4),
          }
        })
        .collect::<Vec<_>>();
      Template::render("index", ImagesList { images: image_list })
    }
    Err(_) => Template::render("index", ImagesList { images: vec![] }),
  }
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
  let queries = [
    r#"
        CREATE TABLE IF NOT EXISTS images (
            id TEXT PRIMARY KEY DEFAULT gen_random_uuid(), 
            path VARCHAR(150) NOT NULL,
            width INTEGER NOT NULL, 
            height INTEGER NOT NULL, 
            title VARCHAR(200) NOT NULL, 
            private BOOLEAN DEFAULT FALSE, 
            content bytea NOT NULL )
            "#,
    r#"
        CREATE INDEX IF NOT EXISTS images_search_index 
            ON images 
            USING gin(to_tsvector('simple', title))
            "#,
    r#"
        CREATE TABLE IF NOT EXISTS comments (
            image_id TEXT, 
            user_name VARCHAR(150), 
            comment VARCHAR(300), 
            CONSTRAINT fk_comment_image 
            FOREIGN KEY (image_id) REFERENCES images(id))
            "#,
    r#"
        CREATE TABLE IF NOT EXISTS metadata (
            image_id TEXT, 
            creationTime FLOAT, 
            camera_make VARCHAR(10000), 
            camera_model VARCHAR (10000), 
            orientation INTEGER, 
            horizontal_ppi INTEGER, 
            vertical_ppi INTEGER, 
            shutter_speed FLOAT, 
            color_space VARCHAR(20), 
            CONSTRAINT fk_metadata_images 
            FOREIGN KEY (image_id) REFERENCES images(id))
            "#,
  ];
  let handle = MyPgDatabase::get_one(&rocket)
    .await
    .expect("Database mounted");
  for query in queries {
    handle
      .run(|conn| conn.execute(query, &[]))
      .await
      .expect("Can't initialize the database");
  }
  rocket
}

fn get_database_url() -> String {
  if let Ok(url) = std::env::var("DATABASE_URL") {
    return url;
  } else {
    return String::from("postgres://postgres:rocket@127.0.0.1/postgres");
  }
}

#[launch]
fn rocket() -> _ {
  let db: Map<_, Value> = map! {
      "url" => get_database_url().into()
  };
  let figment = rocket::Config::figment().merge(("databases", map!["postgres" => db]));

  let myshield = shield::Shield::default()
    .enable(shield::Referrer::NoReferrer)
    .enable(shield::Prefetch::Off)
    .enable(shield::XssFilter::Enable)
    .enable(shield::Frame::Deny);

  rocket::custom(figment)
    .attach(MyPgDatabase::fairing())
    .attach(AdHoc::on_ignite("Postgres init", run_migrations))
    .attach(CSP {})
    .attach(Template::fairing())
    .attach(myshield)
    .attach(rocket_csrf::Fairing::default())
    .mount("/static", FileServer::from("static"))
    .mount(
      "/",
      routes![
        images,
        upload,
        upload_csrf,
        upload_post,
        img,
        images_name,
        images_name_csrf,
        comment,
        search,
        error
      ],
    )
}
