use rocket::{routes, Responder};

mod file;
mod get;
mod paste;
mod root;
mod redirect;

pub fn mounts() -> Vec<rocket::Route> {
    /*! Return the list of the application's exposed endpoints */
    routes![file::create, paste::create, redirect::create, get::get, root::root]
}

#[derive(Responder)]
#[response(status = 201)]
struct CreatedResponse(String);
