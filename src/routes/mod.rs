use rocket::{get, response::Responder, routes, State};
use tokio::fs;

use crate::types::{Record, RecordData};

mod file;
mod paste;
mod redirect;

pub fn mounts() -> Vec<rocket::Route> {
    /*! Return the list of the application's exposed endpoints */
    routes![file::upload, get]
}

#[derive(Debug, Responder)]
pub enum RecordResponse {
    #[response(content_type = "binary")]
    File(rocket::tokio::fs::File),
    Redirect(rocket::response::Redirect),
    #[response(content_type = "text/plain")]
    Paste(String),
}

#[get("/<slug>")]
pub async fn get<'r>(
    slug: String,
    redis: &State<redis::Client>,
) -> crate::Result<impl Responder<'r, 'static>> {
    let mut conn = redis.get_async_connection().await?;

    let record = Record::pull(&slug, &mut conn)
        .await?
        .ok_or(crate::Error::NotFound(slug))?;

    log::debug!("Returning {:#?}", record);

    Ok(match record.data() {
        RecordData::File { path, .. } => RecordResponse::File(fs::File::open(path).await?),
        _ => unimplemented!(),
    })
}
