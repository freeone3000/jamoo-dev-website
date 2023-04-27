/** imports */

use std::collections::HashMap;
use std::convert::From;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use iron::{
    headers::*,
    status,
    mime::{Mime, SubLevel, TopLevel},
    prelude::*
};
use router::Router;

/** exports */
#[derive(Clone)] // TODO REMOVE!!
pub struct WebsiteConfig {
    pub site_root: String
}

pub fn serve(config: WebsiteConfig) {
    let mut router = Router::new();

    router.get("/", handle_request_indexed, "template_index");
    router.get("/pages/:template", handle_request_template, "template_non_index");
    router.get("/static/:file", handle_static_file, "static_file");

    let mut chain = Chain::new(router);
    let config_middleware = WebsiteConfigMiddleware {
        config,
    };
    chain.link_before(config_middleware);

    Iron::new(chain).http("0.0.0.0:3000").unwrap();
}

/* middleware */
fn get_config<'a, 'b: 'a>(req: &'a Request<'a, 'b>) -> Option<&'a WebsiteConfig> {
    req.extensions.get::<WebsiteConfig>()
}
struct WebsiteConfigMiddleware {
    config: WebsiteConfig,
}
impl iron::typemap::Key for WebsiteConfig {
    type Value = WebsiteConfig;
}
impl iron::middleware::BeforeMiddleware for WebsiteConfigMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.entry::<WebsiteConfig>().or_insert(self.config.clone());
        Ok(())
    }

    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<()> {
        Err(err)
    }
}


/** actual routing */
fn handle_request_indexed(req: &mut Request) -> IronResult<Response> {
    let config = get_config(req).unwrap();
    return handle_request_backend(config, "index")
}

fn handle_request_template(req: &mut Request) -> IronResult<Response> {
    let config = get_config(req).unwrap();
    let ref template_name = req.extensions.get::<Router>().unwrap().find("template").unwrap_or("/");
    return handle_request_backend(config, template_name)
}

//TODO Precache files, including partials
fn handle_request_backend(config: &WebsiteConfig, template_name: &str) -> IronResult<Response> {
    let template_path_dir = PathBuf::from(format!("{}/pages/", config.site_root));
    let context = mustache::Context::new(template_path_dir);

    let template_path = format!("{}.mustache", template_name);
    let partials: HashMap<String, String> = HashMap::new();

    println!("Rendering template {}", template_path);
    let template;
    match context.compile_path(&template_path) {
        Ok(t) => template = t,
        Err(e) => {
            println!("Error compiling template: {}", e);
            return IronResult::Err(IronError::new(e, status::InternalServerError));
        }
    }
    let body = template.render_to_string(&partials).unwrap();
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
    let config = get_config(req).unwrap();
    let path = format!("{}/static/{}", config.site_root, filename);
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