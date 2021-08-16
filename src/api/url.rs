use rocket::{post, response::Responder, uri, State};

use super::CreatedResponse;
use crate::{
    config::Config,
    types::{HostBase, Record, RecordSettings},
    Error, Result,
};

#[post("/url", data = "<data>")]
pub async fn create<'r>(
    data: Result<String, std::io::Error>,
    host: HostBase<'_>,
    settings: RecordSettings,
    config: &State<Config>,
    redis: &State<redis::Client>,
) -> Result<impl Responder<'r, 'static>> {
    /* If the url data is malformed return an error */
    let url = data.map_err(|err| Error::UrlCreation(err.to_string()))?;

    let mut conn = redis.get_async_connection().await?;

    /* Compute the slug and the appropriate storage path from it */
    let slug = settings.slug(config, &mut conn).await?;

    /* Instanciate a new record from it */
    let record = Record::url(
        rocket::http::uri::Absolute::parse_owned(url)
            .map_err(|err| Error::UrlCreation(err.to_string()))?,
        slug,
        settings.accesses(),
        settings.expiry(),
    );

    log::debug!("Received a new url creation {:?}", record);

    /* Finally try to push the record */
    record.persist(&mut conn).await?;

    log::debug!(
        "Successfully persisted the redirect with the slug `{}`",
        record.slug()
    );

    Ok(CreatedResponse(
        host.with(uri!(super::get::get(slug = record.slug())))
            .to_string(),
    ))
}
