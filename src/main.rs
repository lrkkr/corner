#![forbid(unsafe_code)]

#[macro_use]
extern crate rocket;

use rocket::fs::NamedFile;
use rocket::response;
use rocket::response::{Responder, Response};
use rocket::Request;
use rocket_dyn_templates::Template;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

struct CachedFile(NamedFile);

impl<'r> Responder<'r, 'r> for CachedFile {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.0.respond_to(req)?)
            .raw_header("Cache-control", "max-age=86400")
            .ok()
    }
}

#[derive(Serialize, Deserialize)]
struct Link {
    icon: String,
    title: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct App {
    wan: Vec<Link>,
    lan: Vec<Link>,
}

#[derive(Serialize, Deserialize)]
struct List {
    name: String,
    link: Vec<Link>,
}

#[derive(Serialize, Deserialize)]
struct Mark {
    list: Vec<List>,
}

#[derive(Serialize, Deserialize)]
struct Router {
    app: App,
    mark: Mark,
}

#[get("/")]
fn index() -> Template {
    let content = fs::read_to_string("./Router.toml").expect("Can not read \"Router.toml\"");
    let router: Router = toml::from_str(&content).unwrap();
    Template::render("index", router)
}

#[get("/static/<file..>")]
async fn files(file: PathBuf) -> Option<CachedFile> {
    NamedFile::open(Path::new("static/").join(file))
        .await
        .ok()
        .map(CachedFile)
}

#[rocket::main]
async fn main() {
    if let Err(e) = rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![index, files])
        .launch()
        .await
    {
        println!("Whoops! Rocket didn't launch!");
        drop(e);
    };
}
