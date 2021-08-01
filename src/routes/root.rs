use rocket::{get, response::Responder};

#[get("/")]
pub async fn root<'r>() -> crate::Result<impl Responder<'r, 'static>> {
    /* Include at compile-time to be returned when hitting `/` */
    Ok(include_str!("./root.in"))
}
