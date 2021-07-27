use rocket::{fs::TempFile, post, response::Responder, uri, State};
use tokio::fs;

use super::CreatedResponse;
use crate::{
    config::Config,
    types::{Record, RecordSettings},
    Error, Result,
};

#[post("/file", data = "<file>")]
pub async fn create<'r>(
    file: Result<TempFile<'_>, std::io::Error>,
    settings: RecordSettings,
    config: &State<Config>,
    redis: &State<redis::Client>,
) -> Result<impl Responder<'r, 'static>> {
    let mut file = file.map_err(|err| Error::FileUpload(err.to_string()))?;

    let mut conn = redis.get_async_connection().await?;

    /* Compute the slug and the appropriate storage path from it */
    let slug = settings.slug(config, &mut conn).await?;
    let storage = fs::canonicalize(&config.data_dir).await?.join(&slug);
    let size = file.len();

    /* Instanciate a new record from it */
    let record = Record::file(
        storage.clone(),
        size as usize,
        slug,
        settings.accesses(),
        settings.expiry(),
    );

    log::debug!("Received a file upload {:?}", record);

    /* Finally try to persist this file, and push the record */
    file.persist_to(storage).await?;
    record.persist(&mut conn).await?;

    log::debug!(
        "Successfully persisted the file with the slug `{}`",
        record.slug()
    );

    Ok(CreatedResponse(
        uri!(_, super::get(slug = record.slug())).to_string(),
    ))
}
