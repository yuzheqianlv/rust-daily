use anyhow::Result;
use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::Utc;
use rss::{ChannelBuilder, ItemBuilder};
use serde::Deserialize;
use std::{net::SocketAddr, path::Path};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tracing::{info, warn};

use crate::DailyReport;

/// RSS 服务器配置
#[derive(Debug, Clone)]
pub struct RssServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub title: String,
    pub description: String,
    pub language: String,
}

impl Default for RssServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            base_url: "http://127.0.0.1:8080".to_string(),
            title: "Rust Daily 技术日报".to_string(),
            description: "每日精选 Rust 技术新闻和资讯".to_string(),
            language: "zh-CN".to_string(),
        }
    }
}

/// RSS 服务器
pub struct RssServer {
    config: RssServerConfig,
    reports_dir: String,
}

/// RSS feed 查询参数
#[derive(Debug, Deserialize)]
struct RssFeedQuery {
    limit: Option<usize>,
    format: Option<String>,
}

impl RssServer {
    pub fn new(config: RssServerConfig, reports_dir: String) -> Self {
        Self {
            config,
            reports_dir,
        }
    }

    /// 启动 RSS 服务器
    pub async fn start(&self) -> Result<()> {
        let app = self.create_router();
        let addr = SocketAddr::from(([127, 0, 0, 1], self.config.port));
        
        info!("🚀 RSS 服务器启动在 http://{}:{}", self.config.host, self.config.port);
        info!("📡 RSS Feed URL: {}/feed", self.config.base_url);
        info!("📄 API 端点: {}/api/reports", self.config.base_url);

        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }

    /// 创建 Axum 路由
    fn create_router(&self) -> Router {
        let server = self.clone();
        
        Router::new()
            .route("/", get(|| async { "Rust Daily RSS Server" }))
            .route("/feed", get(rss_feed_handler))
            .route("/feed.xml", get(rss_feed_handler))
            .route("/rss", get(rss_feed_handler))
            .route("/api/reports", get(api_reports_handler))
            .route("/health", get(|| async { "OK" }))
            .with_state(server)
            .layer(ServiceBuilder::new())
    }

    /// 加载历史报告
    fn load_reports(&self, limit: Option<usize>) -> Result<Vec<DailyReport>> {
        let reports_dir = Path::new(&self.reports_dir);
        
        if !reports_dir.exists() {
            return Ok(vec![]);
        }

        let mut reports = Vec::new();
        let mut entries: Vec<_> = std::fs::read_dir(reports_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();

        // 按文件名排序（日期）
        entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        let limit = limit.unwrap_or(10).min(50); // 最多50条
        
        for entry in entries.into_iter().take(limit) {
            match std::fs::read_to_string(entry.path()) {
                Ok(content) => {
                    match serde_json::from_str::<DailyReport>(&content) {
                        Ok(report) => reports.push(report),
                        Err(e) => warn!("解析报告文件失败 {:?}: {}", entry.path(), e),
                    }
                }
                Err(e) => warn!("读取报告文件失败 {:?}: {}", entry.path(), e),
            }
        }

        Ok(reports)
    }

    /// 生成 RSS XML
    fn generate_rss(&self, reports: Vec<DailyReport>) -> Result<String> {
        let mut channel = ChannelBuilder::default()
            .title(&self.config.title)
            .link(&self.config.base_url)
            .description(&self.config.description)
            .language(Some(self.config.language.clone()))
            .generator(Some("Rust Daily Generator v1.0".to_string()))
            .pub_date(Utc::now().to_rfc2822())
            .last_build_date(Utc::now().to_rfc2822())
            .ttl("60".to_string()) // 1小时更新一次
            .build();

        let mut items = Vec::new();

        for report in reports {
            // 为每个报告创建一个 RSS item
            let title = format!("【Rust日报】{}", report.date.format("%Y-%m-%d"));
            let link = format!("{}/reports/{}", self.config.base_url, report.date.format("%Y-%m-%d"));
            let guid = format!("rust-daily-{}", report.date.format("%Y-%m-%d"));
            
            // 构建描述内容
            let mut description = String::new();
            description.push_str(&format!("<h2>📈 每日概览</h2>\n"));
            description.push_str(&format!("<p><strong>📰 新闻来源</strong>: {} 条技术资讯</p>\n", report.items.len()));
            description.push_str(&format!("<p><strong>🤖 生成时间</strong>: {}</p>\n", report.date.format("%Y-%m-%d %H:%M:%S UTC")));
            description.push_str("<hr>\n");
            
            // 添加摘要
            description.push_str("<h2>📋 今日摘要</h2>\n");
            description.push_str(&self.markdown_to_html(&report.summary));
            description.push_str("<hr>\n");
            
            // 添加新闻链接列表
            if !report.items.is_empty() {
                description.push_str("<h2>🔗 相关链接</h2>\n<ul>\n");
                for item in &report.items {
                    description.push_str(&format!(
                        "<li><a href=\"{}\" target=\"_blank\">{}</a> - <em>{}</em></li>\n",
                        item.link, item.title, item.source
                    ));
                }
                description.push_str("</ul>\n");
            }
            
            let item = ItemBuilder::default()
                .title(title)
                .link(link)
                .description(description)
                .pub_date(report.date.to_rfc2822())
                .guid(rss::Guid {
                    value: guid,
                    permalink: false,
                })
                .build();

            items.push(item);
        }

        channel.set_items(items);
        Ok(channel.to_string())
    }

    /// 简单的 Markdown 到 HTML 转换
    fn markdown_to_html(&self, markdown: &str) -> String {
        let mut html = markdown.to_string();
        
        // 转换标题
        html = html.replace("### ", "<h3>").replace("\n", "</h3>\n");
        html = html.replace("## ", "<h2>").replace("\n", "</h2>\n");
        html = html.replace("# ", "<h1>").replace("\n", "</h1>\n");
        
        // 转换列表
        let lines: Vec<&str> = html.lines().collect();
        let mut result = Vec::new();
        let mut in_list = false;
        
        for line in lines {
            if line.trim().starts_with("- ") {
                if !in_list {
                    result.push("<ul>".to_string());
                    in_list = true;
                }
                let item_text = line.trim_start_matches("- ").trim();
                result.push(format!("<li>{}</li>", item_text));
            } else {
                if in_list {
                    result.push("</ul>".to_string());
                    in_list = false;
                }
                if !line.trim().is_empty() {
                    result.push(format!("<p>{}</p>", line));
                }
            }
        }
        
        if in_list {
            result.push("</ul>".to_string());
        }
        
        result.join("\n")
    }
}

impl Clone for RssServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            reports_dir: self.reports_dir.clone(),
        }
    }
}

/// RSS feed 处理器
async fn rss_feed_handler(
    axum::extract::State(server): axum::extract::State<RssServer>,
    Query(params): Query<RssFeedQuery>,
) -> impl IntoResponse {
    match server.load_reports(params.limit) {
        Ok(reports) => {
            match server.generate_rss(reports) {
                Ok(rss_xml) => {
                    let mut headers = axum::http::HeaderMap::new();
                    headers.insert(
                        header::CONTENT_TYPE,
                        "application/rss+xml; charset=utf-8".parse().unwrap(),
                    );
                    headers.insert(
                        header::CACHE_CONTROL,
                        "public, max-age=3600".parse().unwrap(), // 缓存1小时
                    );
                    
                    (StatusCode::OK, headers, rss_xml).into_response()
                }
                Err(e) => {
                    warn!("生成 RSS 失败: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "生成 RSS 失败").into_response()
                }
            }
        }
        Err(e) => {
            warn!("加载报告失败: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "加载报告失败").into_response()
        }
    }
}

/// API 报告列表处理器
async fn api_reports_handler(
    axum::extract::State(server): axum::extract::State<RssServer>,
    Query(params): Query<RssFeedQuery>,
) -> impl IntoResponse {
    match server.load_reports(params.limit) {
        Ok(reports) => {
            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/json; charset=utf-8".parse().unwrap(),
            );
            
            let response_data = serde_json::json!({
                "status": "success",
                "count": reports.len(),
                "reports": reports
            });
            
            (StatusCode::OK, headers, serde_json::to_string_pretty(&response_data).unwrap()).into_response()
        }
        Err(e) => {
            warn!("加载报告失败: {}", e);
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("加载报告失败: {}", e)
            });
            (StatusCode::INTERNAL_SERVER_ERROR, serde_json::to_string(&error_response).unwrap()).into_response()
        }
    }
}

/// RSS 服务器配置构建器
impl RssServerConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(host) = std::env::var("RSS_HOST") {
            config.host = host;
        }
        
        if let Ok(port) = std::env::var("RSS_PORT") {
            if let Ok(port) = port.parse() {
                config.port = port;
            }
        }
        
        if let Ok(base_url) = std::env::var("RSS_BASE_URL") {
            config.base_url = base_url;
        } else {
            config.base_url = format!("http://{}:{}", config.host, config.port);
        }
        
        if let Ok(title) = std::env::var("RSS_TITLE") {
            config.title = title;
        }
        
        if let Ok(description) = std::env::var("RSS_DESCRIPTION") {
            config.description = description;
        }
        
        config
    }
}