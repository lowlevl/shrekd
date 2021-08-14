use rocket::{routes, Responder};

mod file;
mod get;
mod paste;
mod url;

pub fn routes() -> Vec<rocket::Route> {
    /*! Return the list of `/` ::api routes */
    routes![file::create, paste::create, url::create, get::get]
}

#[derive(Responder)]
#[response(status = 201)]
struct CreatedResponse(String);
