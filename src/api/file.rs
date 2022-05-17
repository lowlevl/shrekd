use rocket::{
    data::{ByteUnit, Data},
    http::Header,
    put,
    response::Responder,
    uri, State,
};
use tokio::fs;

use super::CreatedResponse;
use crate::{
    config::Config,
    types::{HostBase, Record, RecordSettings},
    Error, Result,
};

#[put("/<filename>", data = "<data>")]
pub async fn create<'r>(
    filename: String,
    data: Data<'r>,
    host: HostBase<'_>,
    settings: RecordSettings,
    config: &State<Config>,
    redis: &State<redis::Client>,
) -> Result<impl Responder<'r, 'static>> {
    let mut conn = redis.get_async_connection().await?;

    /* Compute the slug and the appropriate storage path from it */
    let slug = settings.slug(config, &mut conn).await?;
    let storage = fs::canonicalize(&config.data_dir).await?.join(&slug);
    let file = data
        .open(config.max_file_size * ByteUnit::B)
        .into_file(&storage)
        .await?;

    if !file.is_complete() {
        /* Remove the incomplete file from the storage */
        tokio::fs::remove_file(&storage).await?;

        return Err(Error::TooLarge);
    }

    let file = file.into_inner();

    /* Compute the Record's max age from it's size */
    let size = file.metadata().await?.len();
    let max_age = config.curve()?.compute_for(size);
    let expiry = settings.expiry(Some(max_age)).unwrap(); // <- unwrap here is safe, because the Option conditioned by the `max_age` parameter

    /* Instanciate a new record from it */
    let record = Record::file(
        filename.to_string(),
        storage,
        size as usize,
        slug,
        settings.accesses(),
        Some(expiry),
    );

    tracing::debug!("Received a file upload {:?}", record);

    /* Finally persist this record in Redis */
    record.persist(&mut conn).await?;

    tracing::debug!(
        "Successfully persisted the file with the slug `{}`",
        record.slug()
    );

    Ok(CreatedResponse(
        host.with(uri!(super::get::get(slug = record.slug())))
            .to_string(),
        Header::new("Expiry", expiry.timestamp().to_string()),
    ))
}
