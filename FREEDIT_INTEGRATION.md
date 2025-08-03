# Freedit 论坛集成指南

本文档介绍如何将 Rust Daily 与 Freedit 论坛系统集成，实现自动发布日报功能。

## 🏗️ 集成架构

```
Rust Daily Generator ──HTTP API──> Freedit 论坛
                                       │
                                   自动发布帖子
                                       │
                                   指定的 Inn
```

## ⚙️ 配置步骤

### 1. 环境变量配置

复制并编辑环境变量文件：

```bash
cp .env.example .env
```

在 `.env` 文件中配置 Freedit 集成：

```env
# Freedit 论坛集成
FREEDIT_ENABLED=true
FREEDIT_BASE_URL=http://127.0.0.1:3001
FREEDIT_USERNAME=your_username
FREEDIT_PASSWORD=your_password
FREEDIT_TARGET_INN=rust

# 自动发布设置
AUTO_PUBLISH=false
```

### 2. 论坛账户设置

1. **创建论坛账户**
   - 访问你的 Freedit 论坛实例
   - 注册或登录账户
   - 确保有发帖权限

2. **创建目标 Inn**
   - 创建名为 "rust" 的 Inn（或修改 `FREEDIT_TARGET_INN`）
   - 设置适当的权限和描述

### 3. 权限要求

确保配置的账户具有以下权限：
- ✅ 登录论坛
- ✅ 在目标 Inn 中发帖
- ✅ 创建帖子和设置标签

## 🚀 使用方法

### 手动发布模式

```bash
# 生成日报并发布到论坛
cargo run -- --publish

# 生成日报，保存到文件并发布
cargo run -- --output daily.md --publish

# 使用批量模式生成并发布
cargo run -- --batch-mode --publish
```

### 自动发布模式

设置 `AUTO_PUBLISH=true` 后，每次运行都会自动发布：

```bash
# 自动生成并发布
cargo run
```

### 测试连接

在正式使用前，测试论坛连接：

```bash
cargo run -- --test-connection
```

## 📊 功能特性

### 发布内容格式

自动发布的帖子包含：

```markdown
*本帖由 Rust Daily 自动生成*

---

[AI 生成的日报内容]

---

📰 **新闻来源**: X 条技术资讯
🤖 **生成时间**: 2024-01-15 12:34:56 UTC
⚡ **项目地址**: [Rust Daily Generator](https://github.com/your-repo/rust-daily)
```

### 自动标签

系统会自动为帖子添加标签：
- `rust` - 核心技术标签
- `日报` - 内容类型标签  
- `技术` - 分类标签
- `每日新闻` - 频率标签

### 错误处理

- 🔐 **认证失败**: 检查用户名和密码
- 📍 **Inn 不存在**: 自动检测并提示
- 🌐 **网络错误**: 自动重试和错误提示
- ⚠️ **发布失败**: 保留本地内容，继续后续流程

## 🛠️ 高级配置

### 自定义 Inn

```env
FREEDIT_TARGET_INN=tech-news
```

### 多实例支持

可以配置多个论坛实例：

```env
FREEDIT_BASE_URL=https://your-forum.com
```

### 定时任务集成

结合 cron 实现定时发布：

```bash
# 每日 8:00 自动生成并发布
0 8 * * * cd /path/to/rust-daily && cargo run --release
```

## 🔍 故障排除

### 常见问题

1. **登录失败**
   ```
   Error: 认证失败: 登录失败，用户名或密码错误
   ```
   - 检查用户名和密码是否正确
   - 确认账户未被锁定

2. **Inn 不存在**
   ```
   Error: Inn不存在: rust
   ```
   - 创建对应的 Inn
   - 或修改 `FREEDIT_TARGET_INN` 配置

3. **发帖权限不足**
   ```
   Error: 创建帖子失败，状态码: 403
   ```
   - 检查账户权限
   - 确认在目标 Inn 中有发帖权限

### 调试模式

启用详细日志：

```bash
RUST_LOG=debug cargo run -- --publish
```

## 📈 性能考虑

### API 调用频率

- 登录：按需调用，会话保持
- 发帖：每次发布一次调用
- 检查：轻量级验证调用

### 网络超时

默认超时设置：
- HTTP 请求：30 秒
- 连接超时：自动重试

## 🔒 安全注意事项

1. **密码安全**
   - 使用强密码
   - 定期更换账户密码
   - 不要在代码中硬编码密码

2. **网络安全**
   - 使用 HTTPS（生产环境）
   - 考虑使用反向代理
   - 配置防火墙规则

3. **权限最小化**
   - 只授予必要的发帖权限
   - 定期检查账户活动
   - 监控自动发布日志

## 📚 开发者参考

### API 接口

集成模块提供的主要接口：

```rust
// 创建客户端
let mut client = FreeditClient::new(base_url, username, password);

// 登录
client.login().await?;

// 发布帖子
client.create_post(inn_name, title, content, &tags).await?;

// 发布日报
client.publish_daily_report(&report, inn_name).await?;
```

### 自定义扩展

可以扩展集成功能：

1. **多平台发布**: 添加其他论坛支持
2. **内容定制**: 修改发布格式和标签
3. **通知功能**: 集成消息推送
4. **统计分析**: 记录发布数据

## ✅ 测试清单

发布前检查：

- [ ] 环境变量配置正确
- [ ] 论坛连接测试通过
- [ ] 目标 Inn 存在且可访问
- [ ] 账户权限充足
- [ ] 日报内容生成正常
- [ ] 发布格式符合预期

## 🤝 贡献指南

欢迎贡献改进：

1. **Bug 修复**: 提交 Issue 和 PR
2. **功能增强**: 讨论后实现新特性
3. **文档完善**: 改进使用说明
4. **测试用例**: 增加自动化测试

---

💡 **提示**: 首次使用建议先在测试环境验证配置，确认无误后再应用到生产环境。