#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate rand;
extern crate tera;

use std::io;
use std::io::{Write, Read};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::borrow::Borrow;
use std::error::Error;

use rocket::{Data, State};
use rocket::fairing::AdHoc;
use rocket::response::{NamedFile, Redirect};
use rocket::request::Form;
use rocket_contrib::Template;

use tera::Context;

mod paste_id;
use paste_id::{PasteID, ID_LEN};

struct PublicUrl(String);
const DEFAULT_PUBLIC_URL: &'static str = "http://localhost:8000";


#[derive(Debug, FromForm)]
struct PasteForm {
    #[form(field = "code")]
    text: String
}

#[get("/")]
fn index() -> Template {
    let context = Context::new();
    Template::render("index", &context)
}

// Save a string as a paste and return the URL
fn upload_to_file(paste: String, public_url: State<PublicUrl>) -> io::Result<String> {
    let id = PasteID::new(ID_LEN);

    let path_name = format!("upload/{}", id);
    let path = Path::new(&path_name);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
        Ok(file) => file,
    };
    match file.write_all(paste.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", 
                           display,
                           why.description()),
        Ok(_) => println!("successfully wrote to {}", display),
    }

    Ok(format!("{public_url}/{id}\n", public_url = public_url.0, id = id))
}

#[post("/api", data = "<paste>")]
fn upload_plain(paste: Data, public_url: State<PublicUrl>) -> io::Result<String> {
    let mut buffer = String::new();
    paste.open().read_to_string(&mut buffer)?;
    Ok(upload_to_file(buffer, public_url).unwrap())
}

#[post("/", data = "<paste>")]
fn upload_html(paste: Result<Form<PasteForm>, Option<String>>, public_url: State<PublicUrl>) -> Redirect {
    match paste {
        Ok(f) => {
            let paste_text = f.get().text.clone();
            Redirect::to(upload_to_file(paste_text, public_url).unwrap().borrow())
        },
        Err(Some(_)) => Redirect::to("/404"),
        Err(None) => Redirect::to("/404"),
    }
}

#[get("/<id>")]
fn retrieve(id: PasteID) -> Template {
    let path_name = format!("upload/{}", id);
    let path = Path::new(&path_name);
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}",
                           display,
                           why.description()),
        Ok(file) => file,
    };
    let mut buffer = String::new();
    match file.read_to_string(&mut buffer) {
        Err(why) => panic!("couldn't read {}: {}",
                            display,
                            why.description()),
        Ok(_) => {
            let mut context = Context::new();
            context.add("id", &format!("{}", id));
            context.add("paste", &buffer);
            Template::render("retrieve", &context)
        }
    }
}

#[get("/<id>/raw", rank = 2)]
fn retrieve_raw(id: PasteID) -> Option<File> {
    let filename = format!("upload/{id}", id = id);
    File::open(&filename).ok()
}

#[get("/static/<file..>")]
fn static_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![
            static_files,
            index,
            upload_plain,
            upload_html,
            retrieve,
            retrieve_raw
        ])
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
