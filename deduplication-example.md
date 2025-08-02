# 去重标记功能使用示例

这个文档演示了 Rust Daily 的去重标记功能如何工作。

## 场景演示

### 1. 首次运行
```bash
$ cargo run
2025-08-02T23:30:00.000Z INFO rust_daily: 启动 Rust 日报生成器
2025-08-02T23:30:00.000Z INFO rust_daily: 获取最近 1 天的新闻
2025-08-02T23:30:01.000Z INFO rust_daily::history: 历史记录文件不存在，创建新的记录
2025-08-02T23:30:05.000Z INFO rust_daily: 从 Rust Blog 获取到 3 条新闻
2025-08-02T23:30:08.000Z INFO rust_daily: 从 This Week in Rust 获取到 8 条新闻
2025-08-02T23:30:08.000Z INFO rust_daily: 总共获取到 11 条新闻
2025-08-02T23:30:08.000Z INFO rust_daily::history: 没有发现重复新闻，共 11 条新新闻
2025-08-02T23:30:15.000Z INFO rust_daily::history: 标记 11 条新闻为已处理

【Rust日报】2025-08-02 

This Week in Rust #610
----------------------
...

📊 本次处理统计:
  新处理: 11 条
  总历史: 11 条
```

### 2. 再次运行（自动去重）
```bash
$ cargo run
2025-08-02T23:35:00.000Z INFO rust_daily: 启动 Rust 日报生成器
2025-08-02T23:35:00.000Z INFO rust_daily: 获取最近 1 天的新闻
2025-08-02T23:35:01.000Z INFO rust_daily::history: 加载历史记录，包含 11 条已处理项目
2025-08-02T23:35:05.000Z INFO rust_daily: 从 Rust Blog 获取到 3 条新闻
2025-08-02T23:35:08.000Z INFO rust_daily: 从 This Week in Rust 获取到 8 条新闻
2025-08-02T23:35:08.000Z INFO rust_daily: 总共获取到 11 条新闻
2025-08-02T23:35:08.000Z INFO rust_daily::history: 过滤掉 11 条已处理的重复新闻，剩余 0 条新新闻

所有新闻都已处理过，没有新内容生成日报
使用 --force 参数可以强制处理所有新闻
使用 --stats 查看历史记录统计
```

### 3. 强制模式
```bash
$ cargo run -- --force
2025-08-02T23:40:00.000Z INFO rust_daily: 启动 Rust 日报生成器
2025-08-02T23:40:00.000Z INFO rust_daily: 获取最近 1 天的新闻
2025-08-02T23:40:00.000Z INFO rust_daily: 强制模式：将处理所有新闻，忽略历史记录
...
【Rust日报】2025-08-02 
...
（不会更新历史记录）
```

### 4. 查看统计信息
```bash
$ cargo run -- --stats
📊 历史记录统计:
  总处理数量: 11
  今日处理: 11
  本周处理: 11
  数据源数量: 2
  上次清理: 从未清理
  最早记录: 2025-08-02 23:29:10
```

### 5. 清理历史记录
```bash
$ cargo run -- --cleanup 7
2025-08-02T23:45:00.000Z INFO rust_daily::history: 清理了 0 条超过 7 天的历史记录
清理完成，删除了 0 条过期记录

$ cargo run -- --clear-history
2025-08-02T23:45:30.000Z INFO rust_daily::history: 清空了所有 11 条历史记录
所有历史记录已清空
```

## 历史记录文件

历史记录保存在 `~/.rust-daily/processing_history.json`：

```json
{
  "items": [
    {
      "id": "a1b2c3d4e5f6",
      "title": "Announcing Rust 1.75.0",
      "url": "https://blog.rust-lang.org/2025/08/02/Rust-1.75.0.html",
      "processed_at": "2025-08-02T23:29:10.123456789Z",
      "source": "Rust Blog"
    },
    ...
  ],
  "last_cleanup": null
}
```

## 工作原理

1. **唯一标识生成**: 基于新闻标题和链接 URL 生成哈希 ID
2. **本地存储**: 历史记录持久化保存在用户目录
3. **智能过滤**: 启动时加载历史记录，自动过滤重复项
4. **实时标记**: 处理完成后立即标记为已处理
5. **统计跟踪**: 记录处理时间、来源等元数据

这个系统确保了：
- ✅ 不会重复处理相同的新闻
- ✅ 支持增量更新和定期运行
- ✅ 可以强制重新处理所有内容
- ✅ 提供完整的历史记录管理