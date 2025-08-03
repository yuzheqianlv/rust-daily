# Rust Daily - Rust 技术日报生成器

基于 Rig 库和 DeepSeek API 的自动化 Rust 技术日报生成工具。通过聚合多个 Rust 相关 RSS 源，使用 AI 生成符合 rustcc.cn 日报格式的专业中文技术摘要。

## 功能特点

- 🔄 **自动聚合**: 从多个 Rust 技术 RSS 源自动获取最新资讯
- 🤖 **AI 摘要**: 使用 DeepSeek API 生成专业的中文技术摘要  
- 📝 **格式规范**: 严格按照 rustcc.cn 日报格式生成内容
- 📅 **灵活时间**: 支持获取最近 N 天的新闻内容
- 📊 **智能分类**: 按内容类型和重要性自动分组排序
- 🎯 **智能过滤**: 自动识别和过滤 Rust 相关技术内容
- 📄 **多种输出**: 支持控制台输出或保存到 Markdown 文件
- 🔖 **去重标记**: 自动标记已处理新闻，避免重复生成
- 📈 **历史管理**: 完整的处理历史记录和统计功能
- ⚙️ **处理模式**: 支持单条详细处理和批量整体处理两种模式
- 🌐 **RSS 集成**: 内置 RSS 服务器，支持标准 RSS 2.0 输出
- 🤖 **守护进程**: 支持后台自动运行，每 4 小时生成日报
- 🔗 **论坛集成**: 通过 RSS feeds 与 Freedit 论坛无缝集成

## Freedit 论坛集成

项目支持与 [Freedit](https://github.com/freedit-org/freedit) 论坛系统集成，实现日报自动发布功能：

### 集成特性
- 🚀 **自动发布**: 生成日报后自动发布到指定论坛
- 🔐 **安全认证**: 支持用户名密码登录
- 🏷️ **智能标签**: 自动添加相关技术标签
- ⚡ **连接测试**: 内置连接和权限检测
- 📊 **错误处理**: 完善的异常处理和重试机制

### 快速开始
```bash
# 配置论坛集成
cp .env.example .env
# 编辑 .env 文件设置 FREEDIT_* 相关配置

# 测试连接
cargo run -- --test-connection

# 生成并发布日报
cargo run -- --publish
```

详细配置请参考 [Freedit 集成指南](FREEDIT_INTEGRATION.md)。

## 处理模式

项目支持两种不同的处理模式：

### 单条处理模式（默认）
- **详细摘要**: 逐条处理每个新闻，为每条新闻生成详细的技术摘要
- **技术深度**: AI 有足够的上下文为单条新闻生成深入的技术分析
- **格式多样**: 根据新闻类型（文章、项目、讨论等）使用相应的格式模板
- **处理延迟**: 包含 API 调用间隔以避免频率限制

### 批量处理模式
- **整体概览**: 将所有新闻批量发送给 AI，生成整体日报摘要
- **快速处理**: 单次 API 调用生成整个日报
- **简洁输出**: 每条新闻的摘要相对简短，重点突出要点
- **传统模式**: 类似于早期版本的处理方式

```bash
# 使用单条处理模式（默认）
cargo run

# 明确指定单条处理模式
cargo run -- --single-mode

# 使用批量处理模式
cargo run -- --batch-mode
```

## 输出格式

项目严格按照 rustcc.cn 日报的格式输出，包括：

- **标题格式**: 【Rust日报】YYYY-MM-DD
- **内容结构**: This Week in Rust、重要文章、项目工具、社区讨论
- **格式规范**: 使用标准的 Markdown 格式，支持链接和代码块
- **专业文风**: 技术性强、表达简洁、适合开发者阅读

参考 `sample-output.md` 查看输出示例。

## 快速开始

### 1. 环境准备

```bash
# 获取 DeepSeek API Key
# 访问 https://platform.deepseek.com/ 注册并获取 API Key

# 配置环境变量
cp .env.example .env
# 编辑 .env 文件，设置你的 DEEPSEEK_API_KEY
```

### 2. 运行日报生成器

```bash
# 生成今日 Rust 技术日报（自动去重）
cargo run

# 生成最近 3 天的技术日报
cargo run -- --days 3

# 强制处理所有新闻，忽略历史记录
cargo run -- --force

# 保存日报到文件
cargo run -- --output rust-daily-2024-01-15.md

# 查看帮助信息
cargo run -- --help
```

### 3. 🌐 RSS 服务器模式

```bash
# 启动 RSS 服务器（默认端口 7080）
cargo run -- --serve

# 使用自定义端口
cargo run -- --serve --port 8080

# 访问 RSS feed
curl http://localhost:7080/feed

# 健康检查
curl http://localhost:7080/health
```

### 4. 🤖 守护进程模式

```bash
# 启动守护进程（每 4 小时自动生成日报）
cargo run -- --daemon

# 查看任务执行统计
cargo run -- --task-stats

# 后台运行
nohup cargo run -- --daemon > daemon.log 2>&1 &
```

详细的守护进程使用指南请参考 [守护进程模式指南](DAEMON_MODE_GUIDE.md)。

### 5. 历史记录管理

```bash
# 查看处理统计信息
cargo run -- --stats

# 清理 30 天前的历史记录
cargo run -- --cleanup 30

# 清空所有历史记录
cargo run -- --clear-history
```

## 去重标记系统

项目内置智能去重功能，确保不会重复处理相同的新闻：

### 自动去重
- **唯一标识**: 基于新闻标题和链接生成唯一 ID
- **本地存储**: 历史记录保存在 `~/.rust-daily/processing_history.json`  
- **智能过滤**: 自动过滤已处理的新闻项
- **增量更新**: 只处理新增的新闻内容

### 历史记录管理
- **统计信息**: 查看总处理数量、今日处理、本周处理等
- **定期清理**: 支持清理过期的历史记录
- **强制模式**: 可选择忽略历史记录重新处理
- **完整重置**: 支持清空所有历史记录

### 存储位置
```
~/.rust-daily/
└── processing_history.json    # 处理历史记录
```

## 模板系统

项目内置了专业的日报生成模板 (`template.md`)，确保输出格式与 rustcc.cn 日报保持一致：

### 模板特性
- **格式规范**: 标题分隔线、链接格式、列表标记等严格规范
- **内容结构**: This Week in Rust、文章详解、项目介绍、社区讨论
- **语言风格**: 专业技术写作，中文表达，易于理解
- **智能分组**: 按来源和重要性自动组织内容

### 自定义模板
可以修改 `src/daily_generator.rs` 中的 `get_template()` 函数来自定义输出格式。

## 项目架构

```
rust-daily/
├── src/
│   ├── main.rs              # 主程序入口和格式化
│   ├── config.rs            # RSS 源配置管理
│   ├── rss_parser.rs        # RSS 解析和内容过滤
│   ├── daily_generator.rs   # AI 摘要生成和模板应用
│   └── history.rs           # 历史记录管理和去重
├── template.md              # 日报生成模板
├── sample-output.md         # 输出示例
├── .env.example             # 环境变量模板
└── README.md               # 说明文档
```

## RSS 数据源

项目默认聚合以下 Rust 技术 RSS 源：

- **Rust Official Blog**: https://blog.rust-lang.org/feed.xml
- **This Week in Rust**: https://this-week-in-rust.org/rss.xml  
- **Rust Users Forum**: https://users.rust-lang.org/latest.rss
- **Rust Internals**: https://internals.rust-lang.org/latest.rss
- **Jorge Aparicio's Blog**: https://blog.japaric.io/index.xml

## 技术栈

- **Rig**: LLM 应用开发框架
- **DeepSeek**: AI 摘要生成，支持长文本和复杂格式
- **RSS**: RSS 解析库，支持多种 RSS 格式  
- **Reqwest**: HTTP 客户端，支持代理和重试
- **Chrono**: 日期时间处理
- **Clap**: 命令行参数解析
- **Tokio**: 异步运行时

## 使用示例

### 基本用法
```bash
# 生成今日日报并在控制台显示
cargo run
```

### 高级用法
```bash
# 生成最近一周的日报并保存到文件
cargo run -- --days 7 --output weekly-report.md
```

### 输出示例

```markdown
【Rust日报】2025-08-02 

This Week in Rust #610
----------------------

阅读：https://this-week-in-rust.org/blog/2025/07/30/this-week-in-rust-610/

文章《构建一个简单的哈希图》
--------------

这篇文章介绍了如何用 Rust 语言从头构建一个简单的哈希表...

Eon：简单且友好的配置格式
--------------

Eon 是一种简单易用的配置文件格式，旨在替代 Toml 和 YAML...

--

From 日报小组 Rust Daily
```

## 命令行选项

```
USAGE:
    rust-daily [OPTIONS]

OPTIONS:
    -d, --days <DAYS>      获取最近几天的新闻 (默认: 1)
    -o, --output <FILE>    输出文件路径 (可选，默认输出到控制台)
    -f, --force            强制处理所有新闻，忽略历史记录
        --single-mode      逐条处理新闻，生成更详细的摘要（默认启用）
        --batch-mode       批量处理新闻，生成整体摘要
        --publish          发布日报到 Freedit 论坛
        --test-connection  测试 Freedit 论坛连接
        --stats            显示历史记录统计信息
        --cleanup <DAYS>   清理超过指定天数的历史记录
        --clear-history    清空所有历史记录
    -h, --help             显示帮助信息
    -V, --version          显示版本信息
```

### 使用场景示例

**日常使用（单条处理模式）**:
```bash
# 每日运行，自动去重，生成详细摘要
cargo run
```

**快速浏览（批量处理模式）**:
```bash
# 快速生成整体日报
cargo run -- --batch-mode
```

**一次性批量处理**:
```bash
# 强制处理最近一周的所有新闻（单条模式）
cargo run -- --days 7 --force --output weekly-report.md

# 快速生成一周摘要（批量模式）
cargo run -- --days 7 --batch-mode --output weekly-summary.md
```

**论坛集成使用**:
```bash
# 测试论坛连接
cargo run -- --test-connection

# 生成日报并发布到论坛
cargo run -- --publish

# 批量模式生成并发布
cargo run -- --batch-mode --publish

# 强制处理并发布
cargo run -- --force --publish --output backup.md
```

**维护历史记录**:
```bash
# 查看统计信息
cargo run -- --stats

# 每月清理一次过期记录
cargo run -- --cleanup 30
```

## 开发与贡献

### 自定义 RSS 源

编辑 `src/config.rs` 文件中的 `default_sources` 数组来添加新的 RSS 源：

```rust
RssSource {
    name: "Your Blog Name".to_string(),
    url: "https://yourblog.com/rss.xml".to_string(),
},
```

### 自定义过滤规则

修改 `src/rss_parser.rs` 中的 `is_rust_related` 函数来调整内容过滤逻辑。

### 自定义输出格式

修改 `src/daily_generator.rs` 中的 `get_template` 函数来自定义日报模板。

## 注意事项

- 需要稳定的网络连接来获取 RSS 源
- DeepSeek API 有调用频率限制，请合理使用
- 部分 RSS 源可能需要代理访问
- 生成的摘要质量取决于原始新闻内容的质量
- 模板格式严格按照 rustcc.cn 日报标准

## 许可证

MIT License