use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};
use std::time::Duration;
use rig::client::{ProviderClient, CompletionClient};

use crate::{Config, DailyGenerator, DailyReport, HistoryManager, RssFetcher};

pub struct TaskScheduler {
    scheduler: JobScheduler,
    reports_dir: String,
}

impl TaskScheduler {
    pub async fn new(reports_dir: String) -> Result<Self> {
        let scheduler = JobScheduler::new().await?;
        
        Ok(TaskScheduler {
            scheduler,
            reports_dir,
        })
    }
    
    /// 启动定时任务调度器
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 启动后端定时任务调度器");
        
        // 添加 4 小时间隔的日报生成任务
        self.add_daily_report_job().await?;
        
        // 启动调度器
        self.scheduler.start().await.map_err(|e| anyhow::anyhow!("Failed to start scheduler: {}", e))?;
        
        info!("⏰ 定时任务调度器已启动");
        info!("📅 日报生成任务将每 4 小时运行一次");
        
        // 立即执行一次任务（可选）
        info!("🔄 立即执行首次日报生成...");
        if let Err(e) = self.generate_daily_report().await {
            warn!("首次日报生成失败: {}", e);
        }
        
        // 保持运行
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await; // 每小时检查一次状态
            
            // 检查调度器状态
            if let Err(e) = self.check_scheduler_health().await {
                error!("调度器健康检查失败: {}", e);
            }
        }
    }
    
    /// 添加日报生成任务（每 4 小时运行）
    async fn add_daily_report_job(&mut self) -> Result<()> {
        let reports_dir = self.reports_dir.clone();
        
        // 使用 cron 表达式：每 4 小时运行一次（在每天的 0, 4, 8, 12, 16, 20 点运行）
        let job = Job::new_async("0 0 */4 * * *", move |_uuid, _l| {
            let reports_dir = reports_dir.clone();
            Box::pin(async move {
                info!("⏰ 开始执行定时日报生成任务");
                
                match Self::execute_daily_report_task(reports_dir).await {
                    Ok(report) => {
                        info!("✅ 定时日报生成完成: {}", report.date.format("%Y-%m-%d %H:%M:%S"));
                        info!("📊 生成了 {} 条新闻摘要", report.items.len());
                    }
                    Err(e) => {
                        error!("❌ 定时日报生成失败: {}", e);
                    }
                }
            })
        })?;
        
        self.scheduler.add(job).await.map_err(|e| anyhow::anyhow!("Failed to add job: {}", e))?;
        
        info!("📝 已添加定时任务：每 4 小时生成日报");
        Ok(())
    }
    
    /// 执行日报生成任务的核心逻辑
    async fn execute_daily_report_task(reports_dir: String) -> Result<DailyReport> {
        // 加载配置
        let config = Config::load()?;
        
        // 创建必要的组件
        let rss_fetcher = RssFetcher::new();
        let mut history_manager = HistoryManager::new()?;
        
        // 创建 DeepSeek 客户端
        let deepseek_client = rig::providers::deepseek::Client::from_env();
        let agent = deepseek_client
            .agent("deepseek-chat")
            .preamble("你是 Rust 中文社区的专业技术编辑，负责整理每日 Rust 技术资讯。你需要按照 rustcc.cn 日报的格式和风格，用专业但易懂的中文编写技术日报。重点关注技术细节、实用价值和社区动态，保持客观中性的技术写作风格。")
            .build();
        
        let daily_generator = DailyGenerator::new(agent);
        
        // 获取新闻
        info!("正在获取 RSS 源...");
        let mut all_news = Vec::new();
        
        for rss_url in &config.rss_sources {
            match rss_fetcher.fetch_rss(&rss_url.url, 1).await {
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
            return Err(anyhow::anyhow!("未获取到任何新闻"));
        }
        
        info!("总共获取到 {} 条新闻", all_news.len());
        
        // 过滤已处理的新闻
        let filtered_news = history_manager.filter_unprocessed(all_news);
        
        if filtered_news.is_empty() {
            info!("所有新闻都已处理过，跳过本次生成");
            // 返回一个空的日报而不是错误
            return Ok(DailyReport {
                date: Utc::now(),
                items: vec![],
                summary: "本次检查未发现新内容".to_string(),
            });
        }
        
        // 生成日报
        info!("正在生成日报摘要，处理 {} 条新新闻...", filtered_news.len());
        let daily_report = daily_generator.generate_daily_report(filtered_news.clone(), true).await?;
        
        // 标记新闻为已处理
        history_manager.mark_as_processed(&filtered_news)?;
        
        // 保存报告到 JSON 文件
        std::fs::create_dir_all(&reports_dir)?;
        let report_filename = format!("{}/{}.json", reports_dir, daily_report.date.format("%Y-%m-%d-%H%M"));
        let report_json = serde_json::to_string_pretty(&daily_report)?;
        std::fs::write(&report_filename, report_json)?;
        info!("报告已保存到: {}", report_filename);
        
        Ok(daily_report)
    }
    
    /// 手动触发日报生成
    pub async fn generate_daily_report(&self) -> Result<DailyReport> {
        Self::execute_daily_report_task(self.reports_dir.clone()).await
    }
    
    /// 检查调度器健康状态
    async fn check_scheduler_health(&self) -> Result<()> {
        // 这里可以添加健康检查逻辑
        // 比如检查磁盘空间、网络连接等
        Ok(())
    }
    
    /// 优雅停止调度器
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("🛑 正在停止定时任务调度器...");
        self.scheduler.shutdown().await.map_err(|e| anyhow::anyhow!("Failed to shutdown scheduler: {}", e))?;
        info!("✅ 定时任务调度器已停止");
        Ok(())
    }
    
    /// 获取下次执行时间
    pub async fn next_execution_time(&self) -> Option<DateTime<Utc>> {
        // 这里可以实现获取下次执行时间的逻辑
        // tokio-cron-scheduler 可能需要额外的方法来实现这个功能
        None
    }
    
    /// 获取任务执行统计
    pub async fn get_task_stats(&self) -> TaskStats {
        TaskStats {
            total_executions: 0, // 需要实现统计逻辑
            successful_executions: 0,
            failed_executions: 0,
            last_execution: None,
            next_execution: self.next_execution_time().await,
        }
    }
}

#[derive(Debug)]
pub struct TaskStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub last_execution: Option<DateTime<Utc>>,
    pub next_execution: Option<DateTime<Utc>>,
}

impl TaskStats {
    pub fn display(&self) {
        println!("📊 定时任务统计:");
        println!("  总执行次数: {}", self.total_executions);
        println!("  成功次数: {}", self.successful_executions);
        println!("  失败次数: {}", self.failed_executions);
        
        if let Some(last) = &self.last_execution {
            println!("  上次执行: {}", last.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        
        if let Some(next) = &self.next_execution {
            println!("  下次执行: {}", next.format("%Y-%m-%d %H:%M:%S UTC"));
        }
    }
}