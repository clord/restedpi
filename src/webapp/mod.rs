use std::borrow::Cow;
use warp::{filters::path::Tail, http::Response, Filter, Rejection, Reply};

#[derive(RustEmbed)]
#[folder = "public/"]
struct Asset;


pub extern fn serve(path: &str) -> Result<impl Reply, Rejection> {
  let mime = mime_guess::guess_mime_type(path);
  let asset: Option<Cow<'static, [u8]>> = Asset::get(path);
  let file = asset.ok_or_else(|| warp::reject::not_found())?;

  Ok(Response::builder()
      .header("content-type", mime.to_string())
      .body(file)
    )
}

