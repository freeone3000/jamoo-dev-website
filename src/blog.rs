use std::fs;
use std::path::PathBuf;
use itertools::Itertools;

const POSTS_ROOT: &str = "posts";

pub(crate) struct Post {
    path: PathBuf,
    post_date: std::time::SystemTime,
}

pub(crate) fn newest_posts(limit: usize, start_at: std::time::SystemTime) -> Vec<Post> {
    let path = { // save me from myself
        let mut path = PathBuf::new();
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
            let mut p = path.clone();
            p.push(entry.file_name());
            Post {
                path: p,
                post_date: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            }
        })
        .filter(|post| post.post_date > start_at)
        .sorted_by(|a, b| b.post_date.cmp(&a.post_date))
        .take(limit)
        .collect()
}

pub(crate) fn render(post: &Post) -> Result<String, anyhow::Error> {
    Ok(markdown::to_html(fs::read_to_string(&post.path)?.as_str()))
}