use rocket::fs::{relative, FileServer};

pub fn routes() -> Vec<rocket::Route> {
    /*! Return the list of `/` ::home routes  */
    FileServer::from(relative!("/home")).into()
}
