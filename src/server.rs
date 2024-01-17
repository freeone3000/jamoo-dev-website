/** imports */
use std::collections::HashMap;
use std::convert::From;
use std::path::{PathBuf};

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};

use crate::blog;
use crate::blog::Post;
use crate::config::WebsiteConfig;
use crate::rss_build::generate_rss_doc;

struct ErrorResponse {
    error: String,
}
// best we can do pending https://github.com/rust-lang/rust/issues/31844
impl<E: std::fmt::Debug + std::fmt::Display > From<E> for ErrorResponse {
    fn from(error: E) -> Self {
        ErrorResponse {
            error: format!("Error: {:?}", error)
        }
    }
}
impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response<axum::body::Body> {
        let body = format!("Error: {}", self.error);
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
type Result<T> = std::result::Result<T, ErrorResponse>;

/** wrapper around xml */
struct ChannelResponse(rss::Channel);
impl IntoResponse for ChannelResponse {
    fn into_response(self) -> Response<axum::body::Body> {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/xml".parse().unwrap());
        (headers, self.0.to_string()).into_response()
    }
}

/** entry point */
pub(crate) fn make_app<T: 'static>(config: T) -> axum::Router where T: WebsiteConfig + Clone {
    use tower_http::{services, compression::CompressionLayer};
    use axum::{routing::get, Router};

    Router::new()
        .route("/", get(handle_blog_index_request::<T>))
        .route("/rss.xml", get(handle_rss_request::<T>))
        .route(format!("/{}/:post", blog::POSTS_ROOT).as_str(), get(handle_blog_request::<T>))
        .route("/pages/:template", get(handle_request_template::<T>))
        .nest_service("/static", services::ServeDir::new("static"))
        .with_state(config)
        .layer(CompressionLayer::new())
}

/** actual routing */
async fn handle_rss_request<C: WebsiteConfig>(State(config): State<C>) -> Result<ChannelResponse> {
    let posts = blog::newest_posts(&config.get_site_root(), usize::MAX, std::time::SystemTime::UNIX_EPOCH);
    let rss = generate_rss_doc(&posts)?;
    Ok(ChannelResponse(rss))
}


async fn handle_blog_index_request<C: WebsiteConfig>(State(config): State<C>) -> Result<Html<String>> {
    let posts = blog::newest_posts(&config.get_site_root(), 5, std::time::SystemTime::UNIX_EPOCH);

    let rendered_posts = posts.into_iter()
        .map(|post| blog::render(post).map(|post| post.into()))
        .collect::<std::result::Result<Vec<HashMap<_, _>>, _>>()?;

    let mut params = HashMap::new();
    params.insert("posts".to_string(), rendered_posts);
    handle_request_backend(&config, "blog", Some(params)).await
}

async fn handle_blog_request<C: WebsiteConfig>(State(config): State<C>, Path(post): Path<String>) -> Result<Html<String>> {
    let post = Post::from_path(&post).and_then(|post| blog::render(post))?;

    let mut params: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
    params.insert("posts".to_string(), vec![post.into()]);
    handle_request_backend(&config, "blog", Some(params)).await
}

async fn handle_request_template<C: WebsiteConfig>(State(config): State<C>, Path(template): Path<String>) -> Result<Html<String>> {
    handle_request_backend::<C, String>(&config, &template, None).await
}

async fn handle_request_backend<C, T>(config: &C,
                             template_name: &str,
                             params: Option<HashMap<String, T>>) -> Result<Html<String>>
where
    C: WebsiteConfig,
    T: serde::Serialize,
{
    let template_path_dir = PathBuf::from(format!("{}/templates/", config.get_site_root()));
    let context = mustache::Context::new(template_path_dir);

    let template_path = format!("pages/{}.mustache", template_name);

    let template = context.compile_path(&template_path)?;
    let body = template.render_to_string(&params)?;
    Ok(Html::from(body))
}