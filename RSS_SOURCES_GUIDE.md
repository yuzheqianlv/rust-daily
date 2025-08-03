# RSS 源配置管理指南

Rust Daily 现在支持通过外部配置文件和命令行工具来方便地管理 RSS 源。

## 📁 配置文件

### 配置文件位置
- **文件名**: `rss_sources.toml`
- **位置**: 项目根目录
- **格式**: TOML 格式

### 配置文件结构
```toml
# Rust Daily RSS 源配置文件

[[rss_sources]]
name = "RSS源名称"
url = "https://example.com/feed.xml"

[[rss_sources]]
name = "另一个RSS源"
url = "https://another.com/rss"
```

### 示例配置
```toml
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
name = "Rust中文社区"
url = "https://rustcc.cn/rss"
```

## 🛠️ 命令行管理工具

### 列出当前 RSS 源
```bash
./target/debug/rust-daily --list-sources
```

输出示例：
```
📡 当前配置的 RSS 源:
┌─────┬─────────────────────────────┬─────────────────────────────────────────────────────┐
│ 序号 │           名称              │                     URL                             │
├─────┼─────────────────────────────┼─────────────────────────────────────────────────────┤
│   1 │ Rust Blog                   │ https://blog.rust-lang.org/feed.xml                 │
│   2 │ This Week in Rust           │ https://this-week-in-rust.org/rss.xml               │
│   3 │ Rust Users Forum            │ https://users.rust-lang.org/latest.rss              │
└─────┴─────────────────────────────┴─────────────────────────────────────────────────────┘
```

### 添加新的 RSS 源
```bash
./target/debug/rust-daily --add-source "源名称,RSS链接"
```

实际例子：
```bash
# 添加 Rust 中文社区
./target/debug/rust-daily --add-source "Rust中文社区,https://rustcc.cn/rss"

# 添加个人博客
./target/debug/rust-daily --add-source "我的博客,https://myblog.com/feed.xml"

# 添加游戏开发相关
./target/debug/rust-daily --add-source "Rust GameDev,https://gamedev.rs/news/index.xml"
```

### 命令行参数格式
- **格式**: `"名称,URL"`
- **分隔符**: 英文逗号 `,`
- **引号**: 建议使用双引号包围整个参数
- **名称**: 不能为空，会在日报中显示
- **URL**: 必须是有效的 RSS/Atom feed 地址

## 🚫 错误处理

### 常见错误及解决方法

1. **重复源名称**
   ```
   Error: RSS 源 'Rust中文社区' 已存在
   ```
   **解决**: 使用不同的名称或先手动编辑配置文件

2. **重复 URL**
   ```
   Error: URL 'https://example.com/feed' 已存在
   ```
   **解决**: 检查是否已添加该 RSS 源

3. **格式错误**
   ```
   ❌ 错误的格式！请使用: --add-source "名称,URL"
   ```
   **解决**: 确保使用正确的 `"名称,URL"` 格式

4. **空名称或 URL**
   ```
   ❌ 名称和 URL 都不能为空！
   ```
   **解决**: 确保名称和 URL 都有内容

## 📝 手动编辑配置文件

### 直接编辑 TOML 文件
您也可以直接编辑 `rss_sources.toml` 文件：

```bash
# 使用您喜欢的编辑器
nano rss_sources.toml
# 或
vim rss_sources.toml
# 或
code rss_sources.toml
```

### 配置文件优势
- **批量编辑**: 可以一次性添加多个源
- **注释支持**: 可以添加说明注释
- **版本控制**: 可以纳入 Git 管理
- **备份恢复**: 容易备份和恢复配置

## 🔄 配置生效

### 立即生效
配置更改后，重新运行程序即可生效：

```bash
# 生成日报（使用新配置）
./target/debug/rust-daily --force

# 启动 RSS 服务器（使用新配置）
./target/debug/rust-daily --serve
```

### 配置验证
添加新源后建议先验证：

```bash
# 查看当前配置
./target/debug/rust-daily --list-sources

# 测试生成（不会影响历史记录）
./target/debug/rust-daily --force --output test-report.md
```

## 🌟 推荐的 RSS 源

### Rust 官方和核心
- **Rust Blog**: `https://blog.rust-lang.org/feed.xml`
- **This Week in Rust**: `https://this-week-in-rust.org/rss.xml`
- **Rust Users Forum**: `https://users.rust-lang.org/latest.rss`
- **Rust Internals**: `https://internals.rust-lang.org/latest.rss`

### Rust 社区和中文资源
- **Rust中文社区**: `https://rustcc.cn/rss`
- **Rust Magazine**: `https://rustmagazine.github.io/rust_magazine_2021/feed.xml`

### 知名开发者博客
- **Jorge Aparicio's Blog**: `https://blog.japaric.io/index.xml`
- **Steve Klabnik's Blog**: `https://words.steveklabnik.com/feed.xml`
- **Without Boats**: `https://without.boats/index.xml`
- **Amos's Blog**: `https://fasterthanli.me/index.xml`
- **Baby Steps**: `http://smallcultfollowing.com/babysteps/blog/feeds/all.atom.xml`

### 专业领域
- **Rust GameDev**: `https://gamedev.rs/news/index.xml`

## 💡 最佳实践

1. **定期更新**: 定期检查和更新 RSS 源列表
2. **质量优先**: 选择高质量、更新频繁的 RSS 源
3. **分类管理**: 可以在名称中添加标识符便于分类
4. **测试新源**: 添加新源后先测试是否正常工作
5. **备份配置**: 定期备份 `rss_sources.toml` 文件

## 🔧 故障排除

### RSS 源无法访问
- 检查网络连接
- 验证 RSS URL 是否有效
- 某些源可能需要 User-Agent 或其他头部

### 配置文件损坏
如果配置文件损坏，删除 `rss_sources.toml`，程序会自动创建新的示例文件。

### 性能问题
如果 RSS 源过多导致生成速度慢：
- 减少不活跃的 RSS 源
- 使用 `--days 1` 限制获取天数
- 考虑分批处理

通过这些工具和方法，您可以轻松管理 Rust Daily 的 RSS 源配置，获得更丰富和个性化的技术资讯。