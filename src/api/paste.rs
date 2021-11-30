use rocket::{http::Header, post, response::Responder, uri, State};

use super::CreatedResponse;
use crate::{
    config::Config,
    types::{HostBase, Record, RecordSettings},
    Error, Result,
};

#[post("/paste", data = "<data>")]
pub async fn create<'r>(
    data: Result<Vec<u8>, std::io::Error>,
    host: HostBase<'_>,
    settings: RecordSettings,
    config: &State<Config>,
    redis: &State<redis::Client>,
) -> Result<impl Responder<'r, 'static>> {
    /* If the paste data is malformed return an error */
    let data = data
        .map_err(|err| Error::PasteCreation(err.to_string()))
        .and_then(|bytes| {
            String::from_utf8(bytes).map_err(|err| Error::PasteCreation(err.to_string()))
        })?;

    let mut conn = redis.get_async_connection().await?;

    /* Compute the slug and the appropriate storage path from it */
    let slug = settings.slug(config, &mut conn).await?;

    /* Instanciate a new record from it */
    let record = Record::paste(data, slug, settings.accesses(), settings.expiry(None));

    tracing::debug!("Received a new paste creation {:?}", record);

    /* Finally try to push the record */
    record.persist(&mut conn).await?;

    tracing::debug!(
        "Successfully persisted the paste with the slug `{}`",
        record.slug()
    );

    Ok(CreatedResponse(
        host.with(uri!(super::get::get(slug = record.slug())))
            .to_string(),
        Header::new("Expiry", "-1"),
    ))
}
