use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    IO(#[from] tokio::io::Error),

    #[error("Configuration Error: {0}")]
    Config(#[from] figment::Error),
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for Error {
    fn respond_to(
        self,
        _request: &'r rocket::request::Request<'_>,
    ) -> rocket::response::Result<'o> {
        use rocket::{http::Status, response::Response};
        use std::io::Cursor;

        let status = match self {
            Error::IO(_) => Status::InternalServerError,
            Error::Config(_) => Status::InternalServerError,
        };
        let output = self.to_string();

        Ok(Response::build()
            .status(status)
            .sized_body(output.len(), Cursor::new(output))
            .finalize())
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
