mod server;
mod blog;
mod rss_build;
mod util;
mod config;

#[tokio::main]
async fn main() {
    let config = config::MemoryWebsiteConfig {
        site_root: String::from("."),
    };
    let router = server::make_app(config);

    let listen_address = std::env::var("LISTEN_ADDRESS").unwrap_or(String::from("0.0.0.0:3000"));
    let listener = tokio::net::TcpListener::bind(listen_address).await.unwrap();

    axum::serve(listener, router).await.unwrap();
}