use rocket::{post, response::Responder, uri, State};

use super::CreatedResponse;
use crate::{
    config::Config,
    types::{Record, RecordSettings},
    Error, Result,
};

#[post("/paste", data = "<data>")]
pub async fn create<'r>(
    data: Result<Vec<u8>, std::io::Error>,
    settings: RecordSettings,
    config: &State<Config>,
    redis: &State<redis::Client>,
) -> Result<impl Responder<'r, 'static>> {
    /* If the paste data is malformed return an error */
    let data = data
        .map_err(|err| Error::PasteUpload(err.to_string()))
        .and_then(|bytes| {
            String::from_utf8(bytes).map_err(|err| Error::PasteUpload(err.to_string()))
        })?;

    let mut conn = redis.get_async_connection().await?;

    /* Compute the slug and the appropriate storage path from it */
    let slug = settings.slug(&config, &mut conn).await?;

    /* Instanciate a new record from it */
    let record = Record::paste(data, slug.clone(), settings.accesses(), settings.expiry());

    log::debug!("Received a new paste creation {:?}", record);

    /* Finally try to persist this file, and push the record */
    record.push(&mut conn).await?;

    log::debug!("Successfully persisted the paste with the slug `{}`", slug);

    Ok(CreatedResponse(
        uri!(_, super::get(slug = slug)).to_string(),
    ))
}
