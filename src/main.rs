mod server;

extern crate iron;
extern crate mustache;
extern crate router;


fn main() {
    let config = server::WebsiteConfig {
        site_root: String::from("."),
    };
    server::serve(config);
}