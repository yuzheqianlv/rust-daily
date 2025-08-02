use anyhow::Result;
use chrono::Utc;
use rig::completion::Prompt;
use rig::providers::deepseek;
use tracing::{debug, info};

use crate::{DailyReport, NewsItem};

#[derive(Debug, Clone)]
pub struct ProcessedNewsItem {
    pub original_item: NewsItem,
    pub ai_summary: String,
}

pub struct DailyGenerator {
    agent: rig::agent::Agent<deepseek::CompletionModel>,
}

impl DailyGenerator {
    pub fn new(agent: rig::agent::Agent<deepseek::CompletionModel>) -> Self {
        Self { agent }
    }

    pub async fn generate_daily_report(&self, items: Vec<NewsItem>, use_single_processing: bool) -> Result<DailyReport> {
        if use_single_processing {
            self.generate_daily_report_single_mode(items).await
        } else {
            self.generate_daily_report_batch_mode(items).await
        }
    }

    pub async fn generate_daily_report_single_mode(&self, items: Vec<NewsItem>) -> Result<DailyReport> {
        info!("开始生成日报，共 {} 条新闻", items.len());

        // 按重要性和时间排序
        let mut sorted_items = items;
        sorted_items.sort_by(|a, b| {
            // 首先按来源重要性排序
            let source_priority = |source: &str| match source {
                "Rust Blog" => 0,
                "This Week in Rust" => 1, 
                "Rust Internals" => 2,
                "Rust Users Forum" => 3,
                _ => 4,
            };
            
            let priority_cmp = source_priority(&a.source).cmp(&source_priority(&b.source));
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }
            
            // 然后按时间排序（最新的在前）
            match (a.pub_date, b.pub_date) {
                (Some(a_date), Some(b_date)) => b_date.cmp(&a_date),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        // 限制新闻数量
        let limited_items: Vec<_> = sorted_items.into_iter().take(10).collect();

        // 逐条处理新闻，生成详细摘要
        let mut processed_items = Vec::new();
        for (index, item) in limited_items.iter().enumerate() {
            info!("正在处理第 {}/{} 条新闻: {}", index + 1, limited_items.len(), item.title);
            
            let ai_summary = self.generate_single_item_summary(item).await?;
            processed_items.push(ProcessedNewsItem {
                original_item: item.clone(),
                ai_summary,
            });
            
            // 小延迟避免 API 限流
            if index < limited_items.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        // 生成整体日报摘要
        let overall_summary = self.generate_overall_summary(&processed_items).await?;

        Ok(DailyReport {
            date: Utc::now(),
            items: limited_items,
            summary: self.format_daily_content(&processed_items, &overall_summary),
        })
    }

    async fn generate_single_item_summary(&self, item: &NewsItem) -> Result<String> {
        let template = self.get_single_item_template();
        let mut prompt = String::new();
        
        prompt.push_str(&template);
        prompt.push_str("\n\n请为以下新闻生成详细的技术摘要：\n\n");
        prompt.push_str(&format!("标题: {}\n", item.title));
        prompt.push_str(&format!("链接: {}\n", item.link));
        prompt.push_str(&format!("来源: {}\n", item.source));
        
        if !item.description.is_empty() {
            prompt.push_str(&format!("描述: {}\n", item.description));
        }
        
        if let Some(date) = item.pub_date {
            prompt.push_str(&format!("发布时间: {}\n", date.format("%Y-%m-%d")));
        }
        
        prompt.push_str("\n请按照模板格式生成这条新闻的详细摘要。");

        debug!("发送单条新闻摘要请求到 DeepSeek");
        
        let summary = self.agent.prompt(&prompt).await?;
        Ok(summary.to_string())
    }

    async fn generate_overall_summary(&self, processed_items: &[ProcessedNewsItem]) -> Result<String> {
        if processed_items.is_empty() {
            return Ok("今日暂无 Rust 相关新闻。".to_string());
        }

        let mut prompt = String::new();
        prompt.push_str("请为今日的 Rust 技术新闻生成一个总体摘要段落（2-3句话），要求：\n");
        prompt.push_str("1. 概括今日主要的技术动态和趋势\n");
        prompt.push_str("2. 突出最重要的发布、更新或讨论\n");
        prompt.push_str("3. 语言简洁专业，适合技术日报开头\n\n");
        prompt.push_str("今日新闻概览：\n");

        for (i, item) in processed_items.iter().enumerate() {
            prompt.push_str(&format!("{}. {} ({})\n", 
                i + 1, 
                item.original_item.title, 
                item.original_item.source
            ));
        }

        let summary = self.agent.prompt(&prompt).await?;
        Ok(summary.to_string())
    }

    fn format_daily_content(&self, processed_items: &[ProcessedNewsItem], overall_summary: &str) -> String {
        let mut content = String::new();
        
        // 添加整体摘要
        content.push_str(overall_summary);
        content.push_str("\n\n");
        
        // 按类型分组处理
        let mut twir_items = Vec::new();
        let mut blog_items = Vec::new();
        let mut forum_items = Vec::new();
        let mut other_items = Vec::new();
        
        for item in processed_items {
            match item.original_item.source.as_str() {
                "This Week in Rust" => twir_items.push(item),
                "Rust Blog" => blog_items.push(item),
                "Rust Users Forum" | "Rust Internals" => forum_items.push(item),
                _ => other_items.push(item),
            }
        }
        
        // 按重要性顺序添加内容
        if !twir_items.is_empty() {
            self.add_section_content(&mut content, "This Week in Rust", &twir_items);
        }
        if !blog_items.is_empty() {
            self.add_section_content(&mut content, "官方博客", &blog_items);
        }
        if !other_items.is_empty() {
            self.add_section_content(&mut content, "社区项目", &other_items);
        }
        if !forum_items.is_empty() {
            self.add_section_content(&mut content, "社区讨论", &forum_items);
        }
        
        content.push_str("\n--\n\nFrom 日报小组 Rust Daily");
        content
    }
    
    fn add_section_content(&self, content: &mut String, _section_name: &str, items: &[&ProcessedNewsItem]) {
        for item in items {
            content.push_str(&item.ai_summary);
            content.push_str("\n\n");
        }
    }
    
    fn get_single_item_template(&self) -> String {
        r#"你是 Rust 中文社区的专业技术编辑。请为单条 Rust 技术新闻生成详细的摘要内容，按照以下格式：

## 输出格式（根据新闻类型选择）：

### 对于 This Week in Rust:
This Week in Rust #[期号]
----------------------

阅读：[链接]

### 对于技术文章:
文章《[标题]》
--------------

[详细技术分析，包含：]
- 文章主要内容和技术要点
- 核心概念解释和实现细节
- 适用场景和实际价值
- 对 Rust 生态的意义

[Reddit] | 阅读：[原文链接]

### 对于项目/工具:
[项目名]：[简短描述]
--------------

[项目详细介绍：]
- 项目目标和解决的问题
- 主要功能特点（用 * 列表）
- 技术特色和创新点
- 使用示例（如果有）
- 与现有方案的对比

[Reddit] | 仓库：[GitHub链接]

### 对于社区讨论:
讨论：[讨论主题]
-----------------

[讨论要点总结]

"[重要观点引用]"

Reddit：[讨论链接]

## 要求：
1. 使用准确的 Markdown 格式
2. 保持专业但易懂的技术写作风格
3. 突出技术细节和实用价值
4. 每条摘要要有足够的技术深度"#.to_string()
    }

    pub async fn generate_daily_report_batch_mode(&self, items: Vec<NewsItem>) -> Result<DailyReport> {
        info!("开始生成日报（批量模式），共 {} 条新闻", items.len());

        // 按重要性和时间排序
        let mut sorted_items = items;
        sorted_items.sort_by(|a, b| {
            // 首先按来源重要性排序
            let source_priority = |source: &str| match source {
                "Rust Blog" => 0,
                "This Week in Rust" => 1, 
                "Rust Internals" => 2,
                "Rust Users Forum" => 3,
                _ => 4,
            };
            
            let priority_cmp = source_priority(&a.source).cmp(&source_priority(&b.source));
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }
            
            // 然后按时间排序（最新的在前）
            match (a.pub_date, b.pub_date) {
                (Some(a_date), Some(b_date)) => b_date.cmp(&a_date),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        // 限制新闻数量
        let limited_items: Vec<_> = sorted_items.into_iter().take(10).collect();

        // 批量生成摘要
        let batch_summary = self.generate_batch_summary(&limited_items).await?;

        Ok(DailyReport {
            date: Utc::now(),
            items: limited_items,
            summary: batch_summary,
        })
    }

    async fn generate_batch_summary(&self, items: &[NewsItem]) -> Result<String> {
        let template = self.get_batch_template();
        let mut prompt = String::new();
        
        prompt.push_str(&template);
        prompt.push_str("\n\n请为以下新闻生成整体日报摘要：\n\n");
        
        for (i, item) in items.iter().enumerate() {
            prompt.push_str(&format!("{}. 标题: {}\n", i + 1, item.title));
            prompt.push_str(&format!("   链接: {}\n", item.link));
            prompt.push_str(&format!("   来源: {}\n", item.source));
            
            if !item.description.is_empty() {
                prompt.push_str(&format!("   描述: {}\n", item.description));
            }
            
            if let Some(date) = item.pub_date {
                prompt.push_str(&format!("   发布时间: {}\n", date.format("%Y-%m-%d")));
            }
            prompt.push_str("\n");
        }
        
        prompt.push_str("\n请按照模板格式生成今日 Rust 技术日报的整体摘要。");

        debug!("发送批量摘要请求到 DeepSeek");
        
        let summary = self.agent.prompt(&prompt).await?;
        Ok(summary.to_string())
    }

    fn get_batch_template(&self) -> String {
        r#"你是 Rust 中文社区的专业技术编辑。请为今日的 Rust 技术新闻生成一份完整的日报摘要，按照以下格式：

## 输出格式：

This Week in Rust #[期号]（如果有）
----------------------

阅读：[链接]

文章《[标题]》
--------------

[简要技术分析，包含要点和价值]

[项目名]：[简短描述]
--------------

[项目介绍和特点]

讨论：[讨论主题]
-----------------

[讨论要点总结]

## 要求：
1. 将所有新闻整合到一份完整的日报中
2. 按重要性排序：This Week in Rust > 官方博客 > 重要项目 > 社区讨论
3. 每个条目简洁明了，突出技术要点
4. 使用标准 Markdown 格式
5. 保持专业技术写作风格"#.to_string()
    }
}