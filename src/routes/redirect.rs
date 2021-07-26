use rocket::{post, response::Responder, uri, State};

use super::CreatedResponse;
use crate::{
    config::Config,
    types::{Record, RecordSettings},
    Error, Result,
};

#[post("/redirect", data = "<data>")]
pub async fn create<'r>(
    data: Result<String, std::io::Error>,
    settings: RecordSettings,
    config: &State<Config>,
    redis: &State<redis::Client>,
) -> Result<impl Responder<'r, 'static>> {
    /* If the redirect data is malformed return an error */
    let url = data.map_err(|err| Error::RedirectCreation(err.to_string()))?;

    let mut conn = redis.get_async_connection().await?;

    /* Compute the slug and the appropriate storage path from it */
    let slug = settings.slug(config, &mut conn).await?;

    /* Instanciate a new record from it */
    let record = Record::redirect(
        rocket::http::uri::Absolute::parse_owned(url)
            .map_err(|err| Error::RedirectCreation(err.to_string()))?,
        slug.clone(),
        settings.accesses(),
        settings.expiry(),
    );

    log::debug!("Received a new redirect creation {:?}", record);

    /* Finally try to push the record */
    record.push(&mut conn).await?;

    log::debug!(
        "Successfully persisted the redirect with the slug `{}`",
        slug
    );

    Ok(CreatedResponse(
        uri!(_, super::get(slug = slug)).to_string(),
    ))
}
