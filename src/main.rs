#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate rand;
extern crate tera;

use std::io;
use std::path::{Path, PathBuf};
use std::fs::File;

use rocket::{Data, State};
use rocket::fairing::AdHoc;
use rocket::response::NamedFile;
use rocket_contrib::Template;

use tera::Context;

mod paste_id;
use paste_id::{PasteID, ID_LEN};

struct PublicUrl(String);
const DEFAULT_PUBLIC_URL: &'static str = "http://localhost:8000";

#[get("/")]
fn index() -> Template {
    let context = Context::new();
    Template::render("index", &context)
}

#[post("/", data = "<paste>")]
fn upload(paste: Data, public_url: State<PublicUrl>) -> io::Result<String> {
    let id = PasteID::new(ID_LEN);
    let filename = format!("upload/{id}", id = id);
    let url = format!("{public_url:?}/{id}\n", public_url = public_url.0, id = id);

    // Write the paste out to the file and return the URL.
    paste.stream_to_file(Path::new(&filename))?;
    Ok(url)
}

#[get("/<id>")]
fn retrieve(id: PasteID) -> Option<File> {
    let filename = format!("upload/{id}", id = id);
    File::open(&filename).ok()
}

#[get("/static/<file..>")]
fn static_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}


fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![static_files, index, upload, retrieve])
        .attach(AdHoc::on_attach(|rocket| {
            println!("Adding public_url managed state...");
            let public_url = PublicUrl(String::from(
                rocket.config().get_str("public_url")
                    .unwrap_or(DEFAULT_PUBLIC_URL)
            ));
            Ok(rocket.manage(public_url))
        }))
        .attach(Template::fairing())
}

fn main() {
   rocket().launch();
}
