use rocket::{
    http::{
        self,
        uri::{self, Absolute},
    },
    outcome::Outcome,
    request::{FromRequest, Request},
};

/** Get the `Host` header from the [`Request`] and wrap it */
pub struct HostBase<'r>(pub uri::Reference<'r>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HostBase<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        const DEFAULT_PROTO: &str = "http";

        let hostname = request
            .headers()
            .get_one("X-Forwarded-Host")
            .or_else(|| request.headers().get_one("Host"));
        let proto = request
            .headers()
            .get_one("X-Forwarded-Proto")
            .unwrap_or(DEFAULT_PROTO);

        log::trace!(
            "Received the following hostname `{:?}` and protocol `{}`",
            hostname,
            proto
        );

        match hostname
            .map(|hostname| uri::Authority::parse(hostname))
            .map(|authority| {
                authority.map(|authority| {
                    uri::Reference::parse_owned(format!("{}://{}", proto, authority))
                })
            }) {
            Some(Ok(Ok(base))) => Outcome::Success(Self(base.into_normalized())),
            _ => Outcome::Failure((http::Status::HttpVersionNotSupported, ())),
        }
    }
}

impl HostBase<'_> {
    /** Computes the Absolute path from the [`HostBase`] and the `path` */
    pub fn with(&self, path: uri::Origin<'_>) -> uri::Absolute<'_> {
        Absolute::parse_owned(format!("{}{}", self.0, path))
            .unwrap()
            .into_normalized()
    }

    /** Retrieve the inner [`uri::Reference`] from the [`HostBase`] */
    pub fn into_inner(self) -> uri::Reference<'static> {
        use rocket::http::ext::IntoOwned;

        self.0.into_owned()
    }
}
