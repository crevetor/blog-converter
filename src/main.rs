use anyhow::Error;
use serde::{Serialize, Deserialize};
use clap::Parser;
use std::{fs::File, io::Write, path::PathBuf};
use chrono::{DateTime, Utc};

/// Get posts from an api and create Zola files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL of the root of the posts API
    #[arg(short, long, default_value="https://blogapi.crevetor.org/posts/")]
    posts_url: String,

    /// Directory to put the files in
    output_directory: PathBuf,
}

#[derive(Deserialize, Debug)]
struct User {
    first_name: String,
    last_name: String,
}

#[derive(Deserialize, Debug)]
struct Author {
   user: User,
   photo: String, 
}

#[derive(Deserialize, Debug)]
struct Tag {
    id: i32,
    tag: String,
    description: String,
}

#[derive(Deserialize, Debug)]
struct Post {
    id: i32,
    author: Author,
    tags: Vec<Tag>,
    title: String,
    summary: String,
    #[serde(default)]
    content: String,
    published_date: DateTime<Utc>,

}

#[derive(Serialize)]
struct PostHeader {
    title: String,
    description: String,
    date: String,
    authors: Vec<String>,
    taxonomies: Taxonomies,
    extra: Extras,
}

#[derive(Serialize)]
struct Taxonomies {
    tags: Vec<String>
}

#[derive(Serialize)]
struct Extras {
    author: String,
    summary: String,
}

impl From<Post> for PostHeader {
    fn from(value: Post) -> Self {
        PostHeader {
            title: value.title.clone(),
            description: value.summary.clone(),
            date: value.published_date.to_rfc3339(),
            authors: vec![format!("{} {}", value.author.user.first_name, value.author.user.last_name)],
            taxonomies: Taxonomies { tags: value.tags.iter().map(|tag| tag.tag.clone()).collect() },
            extra: Extras { author: format!("{} {}", value.author.user.first_name, value.author.user.last_name), summary: value.summary.clone() }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Args::parse();
    println!("Parsing {} into {}", cli.posts_url, cli.output_directory.to_str().unwrap());

    let resp = reqwest::get(&cli.posts_url).await?;
    let posts: Vec<Post> = resp.json().await?;

    println!("Found {} posts", posts.len());
    for post in posts {
        let complete_post:Post = reqwest::get(format!("{}{}", cli.posts_url, post.id)).await?.json().await?;
        let mut filepath = cli.output_directory.clone();
        filepath.push(post.title);
        filepath.set_extension("md");

        let content = complete_post.content.replace("```shell", "```bash");

        let mut file = File::create(filepath)?;
        file.write_all(b"+++\n")?;
        file.write_all(toml::to_string(&PostHeader::from(complete_post))?.as_bytes())?;
        file.write_all(b"+++\n")?;
        file.write_all(content.as_bytes())?;
    }

    Ok(())
}
