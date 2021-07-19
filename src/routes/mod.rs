use rocket::routes;

mod share;
mod shorten;
mod utils;

pub fn mounts() -> Vec<rocket::Route> {
    /*! Return the list of the application's exposed endpoints */
    routes![share::get, share::upload]
}
