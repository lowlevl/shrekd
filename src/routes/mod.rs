use rocket::{get, response::Responder, routes, State};
use tokio::fs;

use crate::types::{Record, RecordData};

mod file;
mod paste;
mod redirect;

#[allow(clippy::nonstandard_macro_braces)]
pub fn mounts() -> Vec<rocket::Route> {
    /*! Return the list of the application's exposed endpoints */
    routes![file::create, paste::create, redirect::create, get]
}

#[derive(Responder)]
#[response(status = 201)]
struct CreatedResponse(String);

#[allow(clippy::large_enum_variant)]
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

    /* FIXME: There's a race condition here with the garbage-collector */
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    log::debug!("Returning {:#?}", record);

    Ok(match record.data() {
        RecordData::File { path, .. } => RecordResponse::File(fs::File::open(path).await?),
        RecordData::Redirect { to } => {
            RecordResponse::Redirect(rocket::response::Redirect::to(to.clone()))
        }
        RecordData::Paste { body } => RecordResponse::Paste(body.clone()),
    })
}
