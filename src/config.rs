use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssSource {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rss_sources: Vec<RssSource>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // 默认的 Rust 相关 RSS 源
        let default_sources = vec![
            RssSource {
                name: "Rust Blog".to_string(),
                url: "https://blog.rust-lang.org/feed.xml".to_string(),
            },
            RssSource {
                name: "This Week in Rust".to_string(),
                url: "https://this-week-in-rust.org/rss.xml".to_string(),
            },
            RssSource {
                name: "Rust Users Forum".to_string(),
                url: "https://users.rust-lang.org/latest.rss".to_string(),
            },
            RssSource {
                name: "Rust Internals".to_string(),
                url: "https://internals.rust-lang.org/latest.rss".to_string(),
            },
            RssSource {
                name: "Jorge Aparicio's Blog".to_string(),
                url: "https://blog.japaric.io/index.xml".to_string(),
            },
        ];

        Ok(Config {
            rss_sources: default_sources,
        })
    }
}