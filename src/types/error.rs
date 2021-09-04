use std::borrow::Cow;
use thiserror::Error;

/** Represent's an application's error */
#[derive(Error, Debug)]
pub enum Error<'s> {
    /* 4xx errors */
    #[error("Couldn't find the record identified with the slug `{0}`")]
    NotFound(String),

    #[error("File upload failed ({0})")]
    FileUpload(String),

    #[error("Paste creation failed ({0})")]
    PasteCreation(String),

    #[error("Url record creation failed ({0})")]
    UrlCreation(String),

    /* 5xx errors */
    #[error("There was an infortuate error in the application's logic ({0})")]
    Intrinsics(Cow<'s, str>),

    #[error("There's an error with the configuration ({0})")]
    Config(#[from] figment::Error),

    #[error("There was a templating error while generating the UI ({0})")]
    Templating(#[from] liquid::Error),

    #[error("I/O error: {0}")]
    IO(#[from] tokio::io::Error),

    #[error("Could not query the Redis server ({0})")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization or deserialization error ({0})")]
    SerDe(#[from] bincode::Error),
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for Error<'o> {
    fn respond_to(self, req: &'r rocket::request::Request<'_>) -> rocket::response::Result<'o> {
        use rocket::{http::Status, response::status, serde::json};
        use serde::Serialize;

        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        let error = json::Json(ErrorResponse {
            message: self.to_string(),
        });

        Ok(match self {
            /* 4xx errors */
            Error::NotFound(_) => status::NotFound(error).respond_to(req)?,
            Error::FileUpload(_) | Error::PasteCreation(_) | Error::UrlCreation(_) => {
                status::Custom(Status::UnprocessableEntity, error).respond_to(req)?
            }

            /* 5xx errors */
            Error::Redis(_) => status::Custom(Status::ServiceUnavailable, error).respond_to(req)?,
            Error::Config(_)
            | Error::IO(_)
            | Error::SerDe(_)
            | Error::Templating(_)
            | Error::Intrinsics(_) => {
                status::Custom(Status::InternalServerError, error).respond_to(req)?
            }
        })
    }
}

/** Convenience alias of [`std::result::Result`] with the [`Error`] prefilled */
pub type Result<T, E = Error<'static>> = std::result::Result<T, E>;
