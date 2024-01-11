mod server;
mod blog;
mod rss_build;
mod util;

extern crate iron;
extern crate mustache;
extern crate router;


fn main() {
    let config = server::WebsiteConfig {
        site_root: String::from("."),
        listen_address: std::env::var("LISTEN_ADDRESS").unwrap_or(String::from("0.0.0.0:3000")),
    };
    server::serve(config);
}