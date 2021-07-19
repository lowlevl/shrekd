/*!
 * # shrt
 * SHare and shoRTen, simple file & url sharing service.
 */

mod routes;

#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes::mounts())
}
