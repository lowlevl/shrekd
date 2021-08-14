use rocket::{get, response::Responder, State};
use tokio::fs;

use crate::types::{Record, RecordData};

#[derive(Debug, Responder)]
pub enum RecordResponse {
    #[response(content_type = "binary")]
    File(rocket::tokio::fs::File),
    Url(rocket::response::Redirect),
    #[response(content_type = "text/plain")]
    Paste(String),
}

#[get("/<slug>")]
pub async fn get<'r>(
    slug: String,
    redis: &State<redis::Client>,
) -> crate::Result<impl Responder<'r, 'static>> {
    let mut conn = redis.get_async_connection().await?;

    let record = Record::fetch(&slug, &mut conn)
        .await?
        .ok_or(crate::Error::NotFound(slug))?;

    log::debug!("Found {:#?}", record);

    /* Transform the record's data into the suited response */
    let response = match record.data() {
        RecordData::File { path, .. } => RecordResponse::File(fs::File::open(path).await?),
        RecordData::Url { target } => {
            RecordResponse::Url(rocket::response::Redirect::to(target.clone()))
        }
        RecordData::Paste { body } => RecordResponse::Paste(body.clone()),
    };

    /* Consume the record to update it's access count if required */
    record.consume(&mut conn).await?;

    Ok(response)
}
