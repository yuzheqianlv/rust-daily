use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Arg, Command};
use rig::prelude::*;
use rig::providers::deepseek;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

mod rss_parser;
mod daily_generator;
mod config;
mod history;

use rss_parser::RssFetcher;
use daily_generator::DailyGenerator;
use config::Config;
use history::HistoryManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub pub_date: Option<DateTime<Utc>>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReport {
    pub date: DateTime<Utc>,
    pub items: Vec<NewsItem>,
    pub summary: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 加载环境变量
    dotenvy::dotenv().ok();
    
    // 解析命令行参数
    let matches = Command::new("rust-daily")
        .about("Rust 每日新闻摘要生成器")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("输出文件路径 (可选，默认输出到控制台)")
        )
        .arg(
            Arg::new("days")
                .short('d')
                .long("days") 
                .value_name("DAYS")
                .default_value("1")
                .help("获取最近几天的新闻 (默认: 1)")
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .help("强制处理所有新闻，忽略历史记录")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("stats")
                .long("stats")
                .help("显示历史记录统计信息")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("cleanup")
                .long("cleanup")
                .value_name("DAYS")
                .help("清理超过指定天数的历史记录")
        )
        .arg(
            Arg::new("clear-history")
                .long("clear-history")
                .help("清空所有历史记录")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("single-mode")
                .long("single-mode")
                .help("逐条处理新闻，生成更详细的摘要（默认启用）")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("batch-mode")
                .long("batch-mode")
                .help("批量处理新闻，生成整体摘要")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();
    
    let output_file = matches.get_one::<String>("output");
    let days: u64 = matches.get_one::<String>("days")
        .unwrap()
        .parse()
        .unwrap_or(1);
    let force_mode = matches.get_flag("force");
    let show_stats = matches.get_flag("stats");
    let cleanup_days = matches.get_one::<String>("cleanup");
    let clear_history = matches.get_flag("clear-history");
    let _single_mode = matches.get_flag("single-mode");
    let batch_mode = matches.get_flag("batch-mode");
    
    // 默认使用单条处理模式，除非明确指定批量模式
    let use_single_processing = !batch_mode;
    
    // 初始化历史记录管理器
    let mut history_manager = HistoryManager::new()?;
    
    // 处理管理命令
    if show_stats {
        let stats = history_manager.get_stats();
        stats.display();
        return Ok(());
    }
    
    if let Some(cleanup_days_str) = cleanup_days {
        let cleanup_days: u64 = cleanup_days_str.parse()
            .unwrap_or_else(|_| {
                warn!("无效的清理天数，使用默认值 30");
                30
            });
        let removed_count = history_manager.cleanup_old_records(cleanup_days)?;
        println!("清理完成，删除了 {} 条过期记录", removed_count);
        return Ok(());
    }
    
    if clear_history {
        history_manager.clear_all()?;
        println!("所有历史记录已清空");
        return Ok(());
    }
    
    info!("启动 Rust 日报生成器");
    info!("获取最近 {} 天的新闻", days);
    if force_mode {
        info!("强制模式：将处理所有新闻，忽略历史记录");
    }
    
    // 加载配置
    let config = Config::load()?;
    
    // 创建 RSS 获取器
    let rss_fetcher = RssFetcher::new();
    
    // 创建 DeepSeek 客户端
    let deepseek_client = deepseek::Client::from_env();
    let agent = deepseek_client
        .agent("deepseek-chat")
        .preamble("你是 Rust 中文社区的专业技术编辑，负责整理每日 Rust 技术资讯。你需要按照 rustcc.cn 日报的格式和风格，用专业但易懂的中文编写技术日报。重点关注技术细节、实用价值和社区动态，保持客观中性的技术写作风格。")
        .build();
    
    // 创建日报生成器
    let daily_generator = DailyGenerator::new(agent);
    
    // 获取新闻
    info!("正在获取 RSS 源...");
    let mut all_news = Vec::new();
    
    for rss_url in &config.rss_sources {
        match rss_fetcher.fetch_rss(&rss_url.url, days).await {
            Ok(mut items) => {
                info!("从 {} 获取到 {} 条新闻", rss_url.name, items.len());
                for item in &mut items {
                    item.source = rss_url.name.clone();
                }
                all_news.extend(items);
            }
            Err(e) => {
                warn!("获取 RSS 源 {} 失败: {}", rss_url.name, e);
            }
        }
    }
    
    if all_news.is_empty() {
        println!("未获取到任何新闻，请检查网络连接或 RSS 源配置");
        return Ok(());
    }
    
    info!("总共获取到 {} 条新闻", all_news.len());
    
    // 过滤已处理的新闻（除非使用强制模式）
    let filtered_news = if force_mode {
        all_news
    } else {
        history_manager.filter_unprocessed(all_news)
    };
    
    if filtered_news.is_empty() {
        println!("所有新闻都已处理过，没有新内容生成日报");
        println!("使用 --force 参数可以强制处理所有新闻");
        println!("使用 --stats 查看历史记录统计");
        return Ok(());
    }
    
    // 生成日报
    info!("正在生成日报摘要...");
    if use_single_processing {
        info!("使用单条处理模式，生成详细摘要");
    } else {
        info!("使用批量处理模式，生成整体摘要");
    }
    let daily_report = daily_generator.generate_daily_report(filtered_news.clone(), use_single_processing).await?;
    
    // 标记新闻为已处理（除非是强制模式）
    if !force_mode {
        history_manager.mark_as_processed(&filtered_news)?;
    }
    
    // 输出结果
    if let Some(output_path) = output_file {
        std::fs::write(output_path, format_daily_report(&daily_report))?;
        info!("日报已保存到: {}", output_path);
    } else {
        println!("{}", format_daily_report(&daily_report));
    }
    
    // 显示处理统计
    if !force_mode {
        let stats = history_manager.get_stats();
        println!("\n📊 本次处理统计:");
        println!("  新处理: {} 条", filtered_news.len());
        println!("  总历史: {} 条", stats.total_processed);
    }
    
    Ok(())
}

fn format_daily_report(report: &DailyReport) -> String {
    let mut output = String::new();
    
    // 使用 rustcc.cn 日报的标题格式
    output.push_str(&format!("【Rust日报】{} \n\n", report.date.format("%Y-%m-%d")));
    
    // 直接输出 AI 生成的摘要内容
    output.push_str(&report.summary);
    
    // 如果摘要中没有包含 "From 日报小组" 结尾，则添加
    if !report.summary.contains("From 日报小组") {
        output.push_str("\n\n--\n\nFrom 日报小组 Rust Daily\n");
    }
    
    // 添加生成信息
    output.push_str(&format!("\n\n*Generated at {} by Rust Daily*\n", 
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    output
}
