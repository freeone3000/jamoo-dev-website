use std::fs;
use std::path::PathBuf;
use itertools::Itertools;
use anyhow::{Context, Result};
use crate::util;
use crate::util::system_time_to_date_time;

pub(crate) const POSTS_ROOT: &str = "posts";

pub(crate) struct Post {
    pub(crate) title: String,
    pub(crate) path: PathBuf,
    pub(crate) original_fn: std::ffi::OsString,
    pub(crate) post_date: std::time::SystemTime,
}
impl Post {
    pub(crate) fn from_path(request_path: &str) -> Result<Self> {
        let mut path = PathBuf::from(POSTS_ROOT);
        let decoded_path = util::my_urldecode(request_path.chars())?;
        path.push(String::from_utf8(decoded_path)?);

        let post_date = util::path_time_modified(&path);
        let original_fn = path.file_name().ok_or(anyhow::anyhow!("Cannot convert path from blog request"))?.to_owned();
        let title = {
            let entry_as_str = String::from_utf8_lossy(original_fn.as_encoded_bytes());
            entry_as_str.split('.').next().unwrap_or("Untitled").to_string()
        };
        Ok(Post {
            title,
            path,
            original_fn,
            post_date,
        })
    }

    pub(crate) fn url_path(&self) -> String {
        let bytes = (*self.original_fn).as_encoded_bytes();
        let s = util::my_urlencode(&bytes);
        POSTS_ROOT.to_string() + "/" + &s
    }
}

/// Will be sorted by date.
pub(crate) fn newest_posts(site_root: &str, limit: usize, start_at: std::time::SystemTime) -> Vec<Post> {
    let path = { // save me from myself
        let mut path = PathBuf::new();
        path.push(site_root);
        path.push(POSTS_ROOT);
        path
    };

    fs::read_dir(POSTS_ROOT).unwrap()
        // remove errors
        .filter_map(|entry| entry.ok())
        // remove no metadata
        .filter_map(|entry| entry.metadata().ok().and_then(|metadata| Some((entry, metadata))))
        // remove non-files
        .filter(|(_, metadata)| metadata.is_file())
        // map to post
        .map(|(entry, metadata)| {
            let original_fn = entry.file_name().to_owned();

            let mut p = path.clone();
            p.push(entry.file_name());

            let title = {
                let entry_fn = entry.file_name();
                let entry_as_str = String::from_utf8_lossy(entry_fn.as_encoded_bytes());
                entry_as_str.split('.').next().unwrap_or("Untitled").to_string()
            };
            Post {
                title,
                original_fn,
                path: p,
                post_date: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            }
        })
        .filter(|post| post.post_date > start_at)
        .sorted_by(|a, b| b.post_date.cmp(&a.post_date))
        .take(limit)
        .collect()
}

pub(crate) struct RenderedPost {
    pub(crate) title: String,
    pub(crate) post_date: chrono::DateTime<chrono::Utc>,
    pub(crate) body: String,
}
/// Specifically for mustache templates
impl From<RenderedPost> for std::collections::HashMap<String, String> {
    fn from(value: RenderedPost) -> Self {
        let mut map = std::collections::HashMap::new();
        map.insert("title".to_string(), value.title);
        map.insert("body".to_string(), value.body);
        map.insert("post_date".to_string(), value.post_date.to_string());
        map
    }
}

pub(crate) fn render(post: Post) -> Result<RenderedPost> {
    use markdown::{Options, ParseOptions, CompileOptions};
    let options = Options {
        parse: ParseOptions::default(),
        compile: CompileOptions {
            allow_dangerous_html: true,
            allow_dangerous_protocol: true,
            ..CompileOptions::default()
        }
    };
    let file_contents = fs::read_to_string(&post.path).with_context(|| format!("Reading path: {:#?}", post.path))?;
    let body = markdown::to_html_with_options(&file_contents, &options)
        .map_err(|e| anyhow::anyhow!("Failed to render post {}: {}", post.path.display(), e))?;

    Ok(RenderedPost {
        body,
        title: post.title,
        post_date: system_time_to_date_time(post.post_date).ok_or(anyhow::anyhow!("Failed to convert post date"))?,
    })
}
