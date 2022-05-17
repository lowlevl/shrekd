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

#[post("/url", data = "<data>")]
pub async fn create<'r>(
    data: Data<'r>,
    host: HostBase<'_>,
    settings: RecordSettings,
    config: &State<Config>,
    redis: &State<redis::Client>,
) -> Result<impl Responder<'r, 'static>> {
    /* If the url data is malformed return an error */
    let url = data
        .open(config.max_url_size * ByteUnit::B)
        .into_string()
        .await?;

    if !url.is_complete() {
        return Err(Error::TooLarge);
    }

    let url = rocket::http::uri::Absolute::parse_owned(url.into_inner())
        .map_err(|err| Error::UrlCreation(err.to_string()))?;

    /* If the scheme is not `http` or `https`, throw an Err */
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(Error::UrlCreation(
            "The only authorized schemes are `http` and `https`".to_string(),
        ));
    }

    /* If the authority is not present, or has no host, throw an Err */
    if url
        .authority()
        .map(|authority| authority.host().is_empty())
        .unwrap_or(true)
    {
        return Err(Error::UrlCreation(
            "The url must contain at least a scheme and an authority".to_string(),
        ));
    }

    let mut conn = redis.get_async_connection().await?;

    /* Compute the slug and the appropriate storage path from it */
    let slug = settings.slug(config, &mut conn).await?;

    /* Instanciate a new record from it */
    let record = Record::url(url, slug, settings.accesses(), settings.expiry(None));

    tracing::debug!("Received a new url creation {:?}", record);

    /* Finally try to push the record */
    record.persist(&mut conn).await?;

    tracing::debug!(
        "Successfully persisted the redirect with the slug `{}`",
        record.slug()
    );

    Ok(CreatedResponse(
        host.with(uri!(super::get::get(slug = record.slug())))
            .to_string(),
        Header::new("Expiry", "-1"),
    ))
}
