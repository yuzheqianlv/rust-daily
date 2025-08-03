#!/bin/bash

# RSS 和 Freedit 集成测试脚本

echo "🚀 开始 RSS 和 Freedit 集成测试"

# 确保 RSS 服务器正在运行
echo "📡 检查 RSS 服务器状态..."
if curl -s -f http://127.0.0.1:7080/health > /dev/null; then
    echo "✅ RSS 服务器运行正常"
else
    echo "❌ RSS 服务器未运行，请先启动"
    exit 1
fi

# 检查 Freedit 论坛状态
echo "🌐 检查 Freedit 论坛状态..."
if curl -s -f http://127.0.0.1:3001 > /dev/null; then
    echo "✅ Freedit 论坛运行正常"
else
    echo "❌ Freedit 论坛未运行，请先启动"
    exit 1
fi

# 生成一个新的日报以创建新内容
echo "📰 生成新的日报内容..."
./target/debug/rust-daily --force > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ 日报生成成功"
else
    echo "❌ 日报生成失败"
    exit 1
fi

# 检查 RSS feed 是否包含新内容
echo "🔍 检查 RSS feed 内容..."
RSS_CONTENT=$(curl -s http://127.0.0.1:7080/feed)
if echo "$RSS_CONTENT" | grep -q "【Rust日报】"; then
    echo "✅ RSS feed 包含日报内容"
    # 显示 RSS 条目数量
    ITEM_COUNT=$(echo "$RSS_CONTENT" | grep -c "<item>")
    echo "📊 RSS feed 包含 $ITEM_COUNT 个条目"
else
    echo "❌ RSS feed 不包含日报内容"
    exit 1
fi

# 检查 Freedit 中 dev inn 的帖子
echo "🏛️ 检查 Freedit dev inn 现有帖子..."
FREEDIT_POSTS=$(curl -s http://127.0.0.1:3001/inn/dev)
POST_COUNT_BEFORE=$(echo "$FREEDIT_POSTS" | grep -c "list-item-title")
echo "📊 当前 dev inn 有 $POST_COUNT_BEFORE 个帖子"

echo ""
echo "🎯 RSS 和 Freedit 集成测试完成"
echo ""
echo "📈 测试结果总结:"
echo "   - RSS 服务器: ✅ 运行正常"
echo "   - Freedit 论坛: ✅ 运行正常" 
echo "   - RSS feed 内容: ✅ 包含 $ITEM_COUNT 个日报条目"
echo "   - Dev inn 帖子: $POST_COUNT_BEFORE 个"
echo ""
echo "💡 接下来的步骤:"
echo "   1. RSS 服务器已在 http://127.0.0.1:7080/feed 提供 RSS feed"
echo "   2. Freedit 已配置为自动从该 RSS 源创建帖子"
echo "   3. 等待 Freedit 的定时任务运行 (或手动触发)"
echo "   4. 检查 dev inn 是否有新的自动生成的帖子"
echo ""
echo "🔗 相关链接:"
echo "   - RSS Feed: http://127.0.0.1:7080/feed"
echo "   - API 端点: http://127.0.0.1:7080/api/reports"
echo "   - Dev Inn: http://127.0.0.1:3001/inn/dev"
echo "   - Freedit 管理: http://127.0.0.1:3001/admin/view?tree_name=inn_feeds"