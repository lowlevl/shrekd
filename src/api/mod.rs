use rocket::{routes, Responder};

mod file;
mod get;
mod paste;
mod redirect;

pub fn routes() -> Vec<rocket::Route> {
    /*! Return the list of `/` ::api routes */
    routes![file::create, paste::create, redirect::create, get::get]
}

#[derive(Responder)]
#[response(status = 201)]
struct CreatedResponse(String);
