use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

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
        // 尝试从外部配置文件加载
        if let Ok(config) = Self::load_from_file("rss_sources.toml") {
            return Ok(config);
        }
        
        // 如果文件不存在，使用默认配置并创建示例文件
        let config = Self::default_config();
        if let Err(e) = Self::create_example_file() {
            eprintln!("警告: 无法创建示例配置文件: {}", e);
        }
        
        Ok(config)
    }
    
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    fn default_config() -> Self {
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

        Config {
            rss_sources: default_sources,
        }
    }
    
    fn create_example_file() -> Result<()> {
        if Path::new("rss_sources.toml").exists() {
            return Ok(()); // 文件已存在，不覆盖
        }
        
        let example_content = r#"# Rust Daily RSS 源配置文件
# 
# 格式: name = "RSS源名称", url = "RSS链接地址"
# 支持的源类型：RSS 2.0, Atom 1.0
# 
# 配置说明：
# - name: RSS 源的显示名称，会在日报中显示
# - url: RSS feed 的完整 URL 地址
# - 新增源后重新运行程序即可生效
# - 建议添加与 Rust 技术相关的高质量源

[[rss_sources]]
name = "Rust Blog"
url = "https://blog.rust-lang.org/feed.xml"

[[rss_sources]]
name = "This Week in Rust"
url = "https://this-week-in-rust.org/rss.xml"

[[rss_sources]]
name = "Rust Users Forum"
url = "https://users.rust-lang.org/latest.rss"

[[rss_sources]]
name = "Rust Internals"
url = "https://internals.rust-lang.org/latest.rss"

[[rss_sources]]
name = "Jorge Aparicio's Blog"
url = "https://blog.japaric.io/index.xml"

# 添加更多 RSS 源示例（已注释，取消注释即可启用）:

# [[rss_sources]]
# name = "Rust中文社区"
# url = "https://rustcc.cn/rss"

# [[rss_sources]]
# name = "Steve Klabnik's Blog"
# url = "https://words.steveklabnik.com/feed.xml"

# [[rss_sources]]
# name = "Without Boats"
# url = "https://without.boats/index.xml"

# [[rss_sources]]
# name = "Amos's Blog"
# url = "https://fasterthanli.me/index.xml"

# [[rss_sources]]
# name = "Baby Steps"
# url = "http://smallcultfollowing.com/babysteps/blog/feeds/all.atom.xml"

# [[rss_sources]]
# name = "Rust GameDev"
# url = "https://gamedev.rs/news/index.xml"

# [[rss_sources]]
# name = "Rust Magazine"
# url = "https://rustmagazine.github.io/rust_magazine_2021/feed.xml"

# 注意事项：
# 1. 添加新源时请确保 URL 有效
# 2. 建议测试新源是否返回有效的 RSS/Atom 内容
# 3. 过多的源可能影响生成速度
# 4. 某些源可能有访问限制或需要特殊处理
"#;
        
        std::fs::write("rss_sources.toml", example_content)?;
        println!("✅ 已创建示例配置文件: rss_sources.toml");
        println!("💡 您可以编辑此文件来添加或修改 RSS 源");
        
        Ok(())
    }
    
    /// 添加新的 RSS 源到配置文件
    pub fn add_source_to_file(name: &str, url: &str) -> Result<()> {
        let config_path = Path::new("rss_sources.toml");
        
        // 如果文件不存在，先创建
        if !config_path.exists() {
            Self::create_example_file()?;
        }
        
        // 读取现有配置
        let mut config = Self::load_from_file("rss_sources.toml")
            .unwrap_or_else(|_| Self::default_config());
        
        // 检查是否已存在相同的源
        for source in &config.rss_sources {
            if source.name == name {
                return Err(anyhow::anyhow!("RSS 源 '{}' 已存在", name));
            }
            if source.url == url {
                return Err(anyhow::anyhow!("URL '{}' 已存在", url));
            }
        }
        
        // 添加新源
        config.rss_sources.push(RssSource {
            name: name.to_string(),
            url: url.to_string(),
        });
        
        // 序列化并写入文件
        let toml_content = toml::to_string_pretty(&config)?;
        
        // 添加注释头部
        let content_with_header = format!(
            "# Rust Daily RSS 源配置文件\n# \n# 格式: name = \"RSS源名称\", url = \"RSS链接地址\"\n# 支持的源类型：RSS 2.0, Atom 1.0\n# \n# 配置说明：\n# - name: RSS 源的显示名称，会在日报中显示\n# - url: RSS feed 的完整 URL 地址\n# - 新增源后重新运行程序即可生效\n# - 建议添加与 Rust 技术相关的高质量源\n\n{}",
            toml_content
        );
        
        std::fs::write("rss_sources.toml", content_with_header)?;
        
        Ok(())
    }
}