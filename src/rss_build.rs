use crate::blog::Post;
use crate::util::system_time_to_date_time;

type Result<T> = std::result::Result<T, anyhow::Error>;

pub(crate) fn generate_rss_doc(posts: &[Post]) -> Result<String> {
    let mut rss = rss::ChannelBuilder::default()
        .title("Behind the Curtain: Solving Your Own Problems with Code")
        .link("https://jamoo.dev")
        .description("Jasmine Moore's blog")
        .build();

    for post in posts {
        let post_date = system_time_to_date_time(post.post_date).map(|dt| dt.to_rfc2822());
        let url_path = post.url_path();

        let item = rss::ItemBuilder::default()
            .title(post.title.clone())
            .link(format!("https://jamoo.dev/{}", url_path))
            .pub_date(post_date)
            .build();
        rss.items.push(item);
    }

    Ok(rss.to_string())
}