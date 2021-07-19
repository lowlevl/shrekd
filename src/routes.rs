use rocket::{routes, get};

#[get("/<slug>")]
fn get<'r>(slug: String) -> impl rocket::response::Responder<'r, 'static> {

}

pub fn mounts() -> Vec<rocket::Route> {
    routes![get]
}
