use rocket::{fs::TempFile, http::Header, put, response::Responder, uri, State};
use tokio::fs;

use super::CreatedResponse;
use crate::{
    config::Config,
    types::{HostBase, Record, RecordSettings},
    Error, Result,
};

#[put("/<filename>", data = "<file>")]
pub async fn create<'r>(
    filename: String,
    file: Result<TempFile<'_>, std::io::Error>,
    host: HostBase<'_>,
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

    /* Compute the Record's max age from it's size */
    let max_age = config.curve()?.compute_for(size);
    let expiry = settings.expiry(Some(max_age)).unwrap(); // <- unwrap here is safe, because the Option conditioned by the `max_age` parameter

    /* Instanciate a new record from it */
    let record = Record::file(
        filename.to_string(),
        storage.clone(),
        size as usize,
        slug,
        settings.accesses(),
        Some(expiry),
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
        host.with(uri!(super::get::get(slug = record.slug())))
            .to_string(),
        Header::new("Expiry", expiry.timestamp().to_string()),
    ))
}
