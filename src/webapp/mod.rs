use std::borrow::Cow;
use warp::{ http::Response, Rejection, Reply};
use mime_guess::from_path;

#[derive(RustEmbed)]
#[folder = "public/"]
struct Asset;


pub extern fn serve(path: &str) -> Result<impl Reply, Rejection> {
  let mime = from_path(path).first_or_octet_stream();
  let asset: Option<Cow<'static, [u8]>> = Asset::get(path);
  let file = asset.ok_or_else(|| warp::reject::not_found())?;

  Ok(Response::builder()
      .header("content-type", mime.to_string())
      .body(file)
    )
}

