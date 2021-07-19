use rocket::{form::Form, fs::TempFile, get, post, response::Responder, State};
use tokio::fs;

use super::utils;
use crate::{config::Config, Result};

#[get("/<_slug>")]
pub fn get<'r>(_slug: String) -> impl Responder<'r, 'static> {}

#[post("/", data = "<file>")]
pub async fn upload<'r>(
    mut file: Form<TempFile<'_>>,
    config: &State<Config>,
    _redis: &State<redis::Client>,
) -> Result<impl Responder<'r, 'static>> {
    /* Compute the slug and the appropriate storage path from it */
    let slug = utils::slug(config.slug_length);
    let storage = fs::canonicalize(&config.data_dir).await?.join(&slug);

    log::debug!(
        "Received an upload request for a file, computed a slug of `{}` and a storage path of `{:?}`",
        slug,
        storage
    );

    /* Finally try to persist this file */
    file.persist_to(storage).await?;

    log::debug!("Successfully persisted the file with slug `{}`", slug);

    Ok(())
}
