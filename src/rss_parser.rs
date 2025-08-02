use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;
use rss::Channel;
use tracing::{debug, warn};

use crate::NewsItem;

pub struct RssFetcher {
    client: Client,
}

impl RssFetcher {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn fetch_rss(&self, url: &str, days: u64) -> Result<Vec<NewsItem>> {
        debug!("正在获取 RSS: {}", url);
        
        // 获取 RSS 内容
        let response = self.client
            .get(url)
            .header("User-Agent", "Rust-Daily/1.0")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("HTTP 错误: {}", response.status()));
        }
        
        let content = response.text().await?;
        
        // 解析 RSS
        let channel = Channel::read_from(content.as_bytes())?;
        
        let cutoff_date = Utc::now() - Duration::days(days as i64);
        let mut items = Vec::new();
        
        for item in channel.items {
            let title = item.title.unwrap_or_default();
            let link = item.link.unwrap_or_default();
            let description = item.description.unwrap_or_default();
            
            // 解析发布日期
            let pub_date = if let Some(date_str) = item.pub_date {
                match DateTime::parse_from_rfc2822(&date_str) {
                    Ok(dt) => Some(dt.with_timezone(&Utc)),
                    Err(_) => {
                        // 尝试其他日期格式
                        match DateTime::parse_from_rfc3339(&date_str) {
                            Ok(dt) => Some(dt.with_timezone(&Utc)),
                            Err(_) => {
                                warn!("无法解析日期: {}", date_str);
                                None
                            }
                        }
                    }
                }
            } else {
                None
            };
            
            // 只包含指定天数内的新闻
            if let Some(date) = pub_date {
                if date < cutoff_date {
                    continue;
                }
            }
            
            // 过滤 Rust 相关内容
            if Self::is_rust_related(&title) || Self::is_rust_related(&description) {
                items.push(NewsItem {
                    title,
                    link,
                    description: Self::clean_html(&description),
                    pub_date,
                    source: String::new(), // 将由调用者设置
                });
            }
        }
        
        Ok(items)
    }

    fn is_rust_related(text: &str) -> bool {
        let text_lower = text.to_lowercase();
        let rust_keywords = [
            "rust", "cargo", "crate", "rustc", "rustup", "wasm", 
            "tokio", "serde", "actix", "axum", "async", "trait",
            "ownership", "borrow", "lifetime", "macro", "unsafe"
        ];
        
        rust_keywords.iter().any(|keyword| text_lower.contains(keyword))
    }

    fn clean_html(text: &str) -> String {
        // 简单的 HTML 标签清理
        let mut result = text
            .replace("<p>", "")
            .replace("</p>", "\n")
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<br />", "\n");
        
        // 移除其他 HTML 标签
        while let Some(start) = result.find('<') {
            if let Some(end) = result[start..].find('>') {
                result.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        }
        
        // 解码 HTML 实体
        result = result
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ");
        
        // 清理多余的空白
        result.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
}