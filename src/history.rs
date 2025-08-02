use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::NewsItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedItem {
    pub id: String,           // 新闻唯一标识 (URL hash)
    pub title: String,        // 新闻标题
    pub url: String,          // 新闻链接
    pub processed_at: DateTime<Utc>, // 处理时间
    pub source: String,       // 来源
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessingHistory {
    pub items: Vec<ProcessedItem>,
    pub last_cleanup: Option<DateTime<Utc>>,
}

pub struct HistoryManager {
    history_file: PathBuf,
    history: ProcessingHistory,
}

impl HistoryManager {
    pub fn new() -> Result<Self> {
        let history_file = Self::get_history_file_path();
        
        // 确保目录存在
        if let Some(parent) = history_file.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let history = Self::load_history(&history_file)?;
        
        Ok(Self {
            history_file,
            history,
        })
    }
    
    fn get_history_file_path() -> PathBuf {
        // 使用用户目录或当前目录下的 .rust-daily 文件夹
        let base_dir = dirs::home_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
            .join(".rust-daily");
        
        base_dir.join("processing_history.json")
    }
    
    fn load_history(path: &PathBuf) -> Result<ProcessingHistory> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let history: ProcessingHistory = serde_json::from_str(&content)
                .unwrap_or_default();
            info!("加载历史记录，包含 {} 条已处理项目", history.items.len());
            Ok(history)
        } else {
            info!("历史记录文件不存在，创建新的记录");
            Ok(ProcessingHistory::default())
        }
    }
    
    fn save_history(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.history)?;
        fs::write(&self.history_file, content)?;
        debug!("保存历史记录到: {:?}", self.history_file);
        Ok(())
    }
    
    /// 生成新闻项的唯一标识
    fn generate_item_id(item: &NewsItem) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        item.link.hash(&mut hasher);
        item.title.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// 过滤掉已处理的新闻项
    pub fn filter_unprocessed(&self, items: Vec<NewsItem>) -> Vec<NewsItem> {
        let processed_ids: HashSet<String> = self.history.items
            .iter()
            .map(|item| item.id.clone())
            .collect();
        
        let original_count = items.len();
        let filtered_items: Vec<NewsItem> = items
            .into_iter()
            .filter(|item| {
                let id = Self::generate_item_id(item);
                !processed_ids.contains(&id)
            })
            .collect();
        
        let filtered_count = filtered_items.len();
        let duplicate_count = original_count - filtered_count;
        
        if duplicate_count > 0 {
            info!("过滤掉 {} 条已处理的重复新闻，剩余 {} 条新新闻", 
                  duplicate_count, filtered_count);
        } else {
            info!("没有发现重复新闻，共 {} 条新新闻", filtered_count);
        }
        
        filtered_items
    }
    
    /// 标记新闻项为已处理
    pub fn mark_as_processed(&mut self, items: &[NewsItem]) -> Result<()> {
        let now = Utc::now();
        
        for item in items {
            let processed_item = ProcessedItem {
                id: Self::generate_item_id(item),
                title: item.title.clone(),
                url: item.link.clone(),
                processed_at: now,
                source: item.source.clone(),
            };
            
            self.history.items.push(processed_item);
        }
        
        info!("标记 {} 条新闻为已处理", items.len());
        self.save_history()?;
        Ok(())
    }
    
    /// 清理过期的历史记录
    pub fn cleanup_old_records(&mut self, days_to_keep: u64) -> Result<usize> {
        let cutoff_date = Utc::now() - Duration::days(days_to_keep as i64);
        let original_count = self.history.items.len();
        
        self.history.items.retain(|item| item.processed_at >= cutoff_date);
        
        let removed_count = original_count - self.history.items.len();
        
        if removed_count > 0 {
            self.history.last_cleanup = Some(Utc::now());
            self.save_history()?;
            info!("清理了 {} 条超过 {} 天的历史记录", removed_count, days_to_keep);
        }
        
        Ok(removed_count)
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> HistoryStats {
        let now = Utc::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap()
            .and_local_timezone(Utc).unwrap();
        let week_start = now - Duration::days(7);
        
        let today_count = self.history.items
            .iter()
            .filter(|item| item.processed_at >= today_start)
            .count();
        
        let week_count = self.history.items
            .iter()
            .filter(|item| item.processed_at >= week_start)
            .count();
        
        let sources: HashSet<String> = self.history.items
            .iter()
            .map(|item| item.source.clone())
            .collect();
        
        HistoryStats {
            total_processed: self.history.items.len(),
            today_processed: today_count,
            week_processed: week_count,
            unique_sources: sources.len(),
            last_cleanup: self.history.last_cleanup,
            oldest_record: self.history.items
                .iter()
                .map(|item| item.processed_at)
                .min(),
        }
    }
    
    /// 清空所有历史记录
    pub fn clear_all(&mut self) -> Result<()> {
        let count = self.history.items.len();
        self.history = ProcessingHistory::default();
        self.save_history()?;
        info!("清空了所有 {} 条历史记录", count);
        Ok(())
    }
    
    /// 搜索历史记录
    pub fn search(&self, query: &str) -> Vec<&ProcessedItem> {
        let query_lower = query.to_lowercase();
        self.history.items
            .iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&query_lower) ||
                item.source.to_lowercase().contains(&query_lower) ||
                item.url.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct HistoryStats {
    pub total_processed: usize,
    pub today_processed: usize,
    pub week_processed: usize,
    pub unique_sources: usize,
    pub last_cleanup: Option<DateTime<Utc>>,
    pub oldest_record: Option<DateTime<Utc>>,
}

impl HistoryStats {
    pub fn display(&self) {
        println!("📊 历史记录统计:");
        println!("  总处理数量: {}", self.total_processed);
        println!("  今日处理: {}", self.today_processed);
        println!("  本周处理: {}", self.week_processed);
        println!("  数据源数量: {}", self.unique_sources);
        
        if let Some(cleanup) = self.last_cleanup {
            println!("  上次清理: {}", cleanup.format("%Y-%m-%d %H:%M:%S"));
        } else {
            println!("  上次清理: 从未清理");
        }
        
        if let Some(oldest) = self.oldest_record {
            println!("  最早记录: {}", oldest.format("%Y-%m-%d %H:%M:%S"));
        }
    }
}