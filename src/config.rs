pub(crate) trait WebsiteConfig: Send + Sync {
    fn get_site_root(&self) -> &str;
}
#[derive(Clone)]
pub(crate) struct MemoryWebsiteConfig {
    pub(crate) site_root: String,
}
impl WebsiteConfig for MemoryWebsiteConfig {
    fn get_site_root(&self) -> &str {
        &self.site_root
    }
}