use rocket::{get, response::Responder, routes};

mod file;
mod paste;
mod redirect;

pub fn mounts() -> Vec<rocket::Route> {
    /*! Return the list of the application's exposed endpoints */
    routes![file::upload, get]
}

#[get("/<_slug>")]
pub fn get<'r>(_slug: String) -> crate::Result<impl Responder<'r, 'static>> {
    Ok(())
}
