use crate::{types::HostBase, Config};
use rocket::{
    data::ByteUnit,
    fs::{relative, FileServer},
    get,
    response::{content, Responder},
    routes, State,
};

#[get("/")]
fn index<'r>(
    config: &State<Config>,
    host: HostBase<'_>,
) -> crate::Result<impl Responder<'r, 'static>> {
    let template = liquid::ParserBuilder::with_stdlib()
        .build()?
        .parse(include_str!("../../ui/index.html"))?;

    let globals = liquid::object!({
        "limits": {
            "file": ByteUnit::from(config.max_file_size).to_string(),
            "paste": ByteUnit::from(config.max_paste_size).to_string(),
            "url": ByteUnit::from(config.max_url_size).to_string(),
        },
        "base": host.into_inner(),
        "version": env!("CARGO_PKG_VERSION")
    });

    Ok(content::Html(template.render(&globals)?))
}

pub fn attach(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    rocket
        /* Attach the template-generated frontpage UI */
        .mount("/", routes![index])
        /* Attach the `/static` routes from the `static` directory */
        .mount("/static", FileServer::from(relative!("ui/static")))
}
