use rocket::{fs::TempFile, put, response::Responder, State};
use tokio::fs;

use crate::{config::Config, types::Record, utils, Result};

#[put("/@", data = "<file>")]
pub async fn upload<'r>(
    mut file: TempFile<'_>,
    config: &State<Config>,
    redis: &State<redis::Client>,
) -> Result<impl Responder<'r, 'static>> {
    let mut conn = redis.get_async_connection().await?;

    /* Compute the slug and the appropriate storage path from it */
    let slug = utils::slug(config.slug_length);
    let storage = fs::canonicalize(&config.data_dir).await?.join(&slug);
    let size = file.len();

    /* Instanciate a new record from it */
    let record = Record::file(storage.clone(), size as usize, slug.clone(), None, None); // TODO: Permit the set of accesses and expiry

    log::debug!("Received a file upload {:?}", record);

    /* Finally try to persist this file, and push the record */
    file.persist_to(storage).await?;
    record.push(&mut conn).await?;

    log::debug!("Successfully persisted the file with the slug `{}`", slug);

    Ok(())
}
