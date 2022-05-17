use rocket::{
    data::{ByteUnit, Data},
    http::Header,
    post,
    response::Responder,
    uri, State,
};

use super::CreatedResponse;
use crate::{
    config::Config,
    types::{HostBase, Record, RecordSettings},
    Error, Result,
};

#[post("/paste", data = "<data>")]
pub async fn create<'r>(
    data: Data<'r>,
    host: HostBase<'_>,
    settings: RecordSettings,
    config: &State<Config>,
    redis: &State<redis::Client>,
) -> Result<impl Responder<'r, 'static>> {
    /* If the paste data is malformed return an error */
    let bytes = data
        .open(config.max_paste_size * ByteUnit::B)
        .into_bytes()
        .await?;

    if !bytes.is_complete() {
        return Err(Error::TooLarge);
    }

    let text = String::from_utf8(bytes.into_inner())
        .map_err(|err| Error::PasteCreation(err.to_string()))?;

    let mut conn = redis.get_async_connection().await?;

    /* Compute the slug and the appropriate storage path from it */
    let slug = settings.slug(config, &mut conn).await?;

    /* Instanciate a new record from it */
    let record = Record::paste(text, slug, settings.accesses(), settings.expiry(None));

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
