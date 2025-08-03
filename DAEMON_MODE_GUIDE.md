# Rust Daily 守护进程模式使用指南

Rust Daily 现在支持守护进程模式，可以在后端自动每 4 小时运行一次，持续生成日报。

## 🤖 守护进程模式概述

### 什么是守护进程模式？
守护进程模式是一个长期运行的后台服务，它会：
- **自动定时执行**: 每 4 小时自动生成一次日报
- **持续监控**: 24/7 运行，无需手动干预
- **智能过滤**: 只处理新内容，避免重复处理
- **错误恢复**: 遇到错误时自动重试，保证服务稳定性

### ⏰ 执行时间表
- **执行间隔**: 每 4 小时运行一次
- **执行时间**: 每天的 0:00, 4:00, 8:00, 12:00, 16:00, 20:00
- **同步策略**: 与 Freedit Inn Feeds 的 4小时10分钟间隔完美配合

## 🚀 启动守护进程模式

### 基本启动
```bash
# 启动守护进程（前台运行，可以看到日志）
./target/debug/rust-daily --daemon
```

### 后台运行
```bash
# 在后台运行守护进程
nohup ./target/debug/rust-daily --daemon > daemon.log 2>&1 &

# 或使用 systemd（推荐用于生产环境）
sudo systemctl start rust-daily-daemon
```

### 停止守护进程
```bash
# 如果是前台运行，使用 Ctrl+C 优雅停止
# 如果是后台运行
pkill -f "rust-daily --daemon"
```

## 📊 监控和统计

### 查看执行统计
```bash
# 查看定时任务执行统计
./target/debug/rust-daily --task-stats
```

输出示例：
```
📊 定时任务统计:
  总执行次数: 12
  成功次数: 10
  失败次数: 2
  上次执行: 2025-08-03 12:00:01 UTC
  下次执行: 2025-08-03 16:00:00 UTC
```

### 日志监控
```bash
# 实时查看日志（如果使用 nohup）
tail -f daemon.log

# 搜索错误日志
grep ERROR daemon.log
```

## 🔧 配置和环境变量

### 环境变量配置
```bash
# 报告保存目录（默认：./reports）
export REPORTS_DIR="/path/to/reports"

# DeepSeek API 密钥
export DEEPSEEK_API_KEY="your-api-key"

# 日志级别
export RUST_LOG="info"
```

### 系统服务配置（systemd）
创建 `/etc/systemd/system/rust-daily-daemon.service`：

```ini
[Unit]
Description=Rust Daily News Generator Daemon
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/path/to/rust-daily
Environment=DEEPSEEK_API_KEY=your-api-key
Environment=REPORTS_DIR=/path/to/reports
Environment=RUST_LOG=info
ExecStart=/path/to/rust-daily/target/release/rust-daily --daemon
Restart=always
RestartSec=30

[Install]
WantedBy=multi-user.target
```

启用系统服务：
```bash
sudo systemctl daemon-reload
sudo systemctl enable rust-daily-daemon
sudo systemctl start rust-daily-daemon
```

## 🔍 故障排除

### 常见问题

1. **API 配额耗尽**
   ```
   ERROR: DeepSeek API rate limit exceeded
   ```
   **解决**: 检查 API 配额，调整执行频率

2. **网络连接失败**
   ```
   ERROR: 获取 RSS 源 失败: network error
   ```
   **解决**: 检查网络连接，确认 RSS 源可访问

3. **磁盘空间不足**
   ```
   ERROR: No space left on device
   ```
   **解决**: 清理磁盘空间，使用 `--cleanup` 参数清理旧报告

### 健康检查
```bash
# 检查进程是否运行
ps aux | grep "rust-daily --daemon"

# 检查最新生成的报告
ls -la reports/ | head -5

# 验证 RSS 服务器是否响应
curl http://localhost:7080/health
```

## 📈 性能优化

### 资源使用建议
- **内存**: 建议至少 512MB 可用内存
- **CPU**: 单核心即可，生成期间会短时间使用较多 CPU
- **网络**: 稳定的互联网连接，建议带宽 ≥ 1Mbps
- **存储**: 预留至少 1GB 空间用于报告存储

### 优化配置
```bash
# 减少 RSS 源数量可提高性能
./target/debug/rust-daily --list-sources

# 定期清理历史数据
./target/debug/rust-daily --cleanup 30  # 保留30天

# 调整并发设置（环境变量）
export TOKIO_WORKER_THREADS=2
```

## 🔄 集成工作流

### 完整的自动化方案
1. **启动守护进程**: `./target/debug/rust-daily --daemon`
2. **启动 RSS 服务器**: `./target/debug/rust-daily --serve` (另一个终端)
3. **配置 Freedit**: 在 Inn Feeds 中添加 `http://localhost:7080/feed`
4. **监控运行**: 定期检查日志和统计信息

### 数据流程
```
RSS 源 → 守护进程(每4小时) → JSON 报告 → RSS 服务器 → Freedit Inn Feeds
```

### 时间同步
- **守护进程生成**: 0:00, 4:00, 8:00, 12:00, 16:00, 20:00
- **Freedit 抓取**: 每4小时10分钟检查一次
- **最大延迟**: 约4小时10分钟

## 🛡️ 安全考虑

### 网络安全
- RSS 服务器默认仅监听 localhost
- 如需外部访问，请配置防火墙规则
- 使用 HTTPS 代理（如 nginx）提供 SSL 支持

### 权限管理
- 使用专用用户运行服务
- 限制文件系统访问权限
- 定期轮换 API 密钥

### 备份策略
```bash
# 备份配置文件
cp rss_sources.toml rss_sources.toml.bak

# 备份重要报告
tar -czf reports-backup-$(date +%Y%m%d).tar.gz reports/
```

## 💡 最佳实践

1. **监控设置**: 配置日志监控和告警
2. **资源限制**: 使用 systemd 限制资源使用
3. **错误处理**: 设置失败重试和通知机制
4. **版本控制**: 将配置文件纳入版本管理
5. **文档维护**: 记录配置变更和运维操作

通过守护进程模式，Rust Daily 可以完全自动化运行，为您提供持续、稳定的 Rust 技术资讯服务！