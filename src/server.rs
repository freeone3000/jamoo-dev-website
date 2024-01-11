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
use log::trace;
use crate::blog;
use crate::blog::Post;
use crate::rss_build::generate_rss_doc;

/** exports */
#[derive(Clone)]
pub struct WebsiteConfig {
    pub site_root: String,
    pub listen_address: String,
}

pub fn serve(config: WebsiteConfig) {
    let listen_address = config.listen_address.clone();

    let mut router = Router::new();

    router.get("/", handle_blog_index_request, "template_blog_index");
    router.get("/rss.xml", handle_rss_request, "rss");

    router.get(format!("/{}/:post", blog::POSTS_ROOT), handle_blog_request, "template_blog_post");
    router.get("/pages/:template", handle_request_template, "template_non_index");
    router.get("/static/:file", handle_static_file, "static_file");

    let mut chain = Chain::new(router);
    let config_middleware = WebsiteConfigMiddleware {
        config,
    };
    chain.link_before(config_middleware);

    Iron::new(chain).http(listen_address).unwrap();
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
fn handle_rss_request(req: &mut Request) -> IronResult<Response> {
    let config = get_config(req).unwrap();

    let posts = blog::newest_posts(&config.site_root, usize::MAX, std::time::SystemTime::UNIX_EPOCH);
    match generate_rss_doc(&posts) {
        Ok(rss) => {
            let mut resp = Response::with((status::Ok, rss));
            resp.headers.set(ContentType(Mime(
                TopLevel::Application,
                SubLevel::Xml,
                vec![]
            )));
            Ok(resp)
        }
        Err(e) => {
            Ok(Response::with((status::InternalServerError, format!("Error generating rss: {}", e))))
        }
    }
}

fn handle_blog_index_request(req: &mut Request) -> IronResult<Response> {
    let config = get_config(req).unwrap();

    let posts = blog::newest_posts(&config.site_root, 5, std::time::SystemTime::UNIX_EPOCH);
    let rendered_posts = posts.into_iter()
        .map(|post| blog::render(post).map(|post| post.into()))
        .collect::<Result<Vec<HashMap<_, _>>, _>>();
    match rendered_posts {
        Ok(rendered_posts) => {
            let mut params = HashMap::new();
            params.insert("posts".to_string(), rendered_posts);
            handle_request_backend(config, "blog", Some(params))
        }
        Err(e) => {
            Ok(Response::with((status::InternalServerError, format!("Error rendering posts: {}", e))))
        }
    }
}

fn handle_blog_request(req: &mut Request) -> IronResult<Response> {
    trace!("Handling blog request");
    let config = get_config(req).unwrap();
    let post_name = req.extensions.get::<Router>().and_then(|r| r.find("post"));
    if let Some(post_name) = post_name {
        let post = Post::from_path(post_name).and_then(|post| blog::render(post));
        match post {
            Ok(rendered_post) => {
                let mut params: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
                params.insert("posts".to_string(), vec![rendered_post.into()]);
                handle_request_backend(config, "blog", Some(params))
            },
            Err(e) => Ok(Response::with((status::InternalServerError, format!("Error rendering post: {}", e))))
        }
    } else {
        handle_blog_index_request(req)
    }
}

fn handle_request_template(req: &mut Request) -> IronResult<Response> {
    let config = get_config(req).unwrap();
    let template_name = req.extensions.get::<Router>().unwrap().find("template").unwrap_or("/");
    return handle_request_backend::<String>(config, template_name, None)
}

fn handle_request_backend<T>(config: &WebsiteConfig,
                             template_name: &str,
                             params: Option<HashMap<String, T>>) -> IronResult<Response>
where
    T: serde::Serialize,
{
    let template_path_dir = PathBuf::from(format!("{}/templates/", config.site_root));
    let context = mustache::Context::new(template_path_dir);

    let template_path = format!("pages/{}.mustache", template_name);

    let template;
    match context.compile_path(&template_path) {
        Ok(t) => template = t,
        Err(e) => {
            println!("Error compiling template at {}: {}", template_path, e);
            return Err(IronError::new(e, status::NotFound));
        }
    }
    let body = template.render_to_string(&params).unwrap();
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