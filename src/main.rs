use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

extern crate iron;
extern crate mustache;
extern crate router;

use iron::{
    headers::*,
    status,
    mime::{Mime, SubLevel, TopLevel},
    prelude::*
};
use router::Router;

fn main() {
    let mut router = Router::new();
    router.get("/", handle_request_indexed, "template_index");
    router.get("/pages/:template", handle_request_template, "template_nonindex");
    router.get("/static/:file", handle_static_file, "static_file");

    Iron::new(router).http("localhost:3000").unwrap();
}

fn handle_request_indexed(_: &mut Request) -> IronResult<Response> {
    return handle_request_backend("index")
}

fn handle_request_template(req: &mut Request) -> IronResult<Response> {
    let ref template_name = req.extensions.get::<Router>().unwrap().find("template").unwrap_or("/");
    return handle_request_backend(template_name)
}

fn handle_request_backend(template_name: &str) -> IronResult<Response> {
    let frontend_path = format!("pages/{}.html", template_name);
    let template_path = format!("templates/{}.html", template_name);

    let body: String;
    if Path::new(&template_path).exists() {
        let mut file = fs::File::open(template_path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let template = mustache::compile_str(&data).unwrap();
        let mut bytes = vec![];
        template.render(&mut bytes, &HashMap::<String, String>::new()).unwrap();
        body = String::from_utf8(bytes).unwrap();
    } else if Path::new(&frontend_path).exists() {
        let mut file = fs::File::open(frontend_path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        body = data;
    } else {
        return Ok(Response::with((status::NotFound, "Template not found")))
    }
    let mut resp = Response::with((status::Ok, body));
    resp.headers.set(ContentType(Mime(
        TopLevel::Text,
        SubLevel::Html,
        vec![]
    )));
    Ok(resp)
}

fn handle_static_file(req: &mut Request) -> IronResult<Response> {
    let ref filename = req.extensions.get::<Router>().unwrap().find("file").unwrap_or("/");
    let path = format!("static/{}", filename);
    if !Path::new(&path).exists() {
        return Ok(Response::with((status::NotFound, "File not found")))
    }

    let mut file = fs::File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let mut resp = Response::with((status::Ok, data));
    resp.headers.set(ContentType(mime_type_for_file(filename)));
    Ok(resp)
}

fn mime_type_for_file(filename: &str) -> Mime {
    if filename.ends_with(".css") {
        Mime(
            TopLevel::Text,
            SubLevel::Css,
            vec![]
        )
    } else {
        Mime(
            TopLevel::Text,
            SubLevel::Plain,
            vec![]
        )
    }
}