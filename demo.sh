#!/bin/bash

# Rust Daily 演示脚本
echo "🦀 Rust Daily - Rust 技术日报生成器演示"
echo "========================================"
echo ""

# 检查环境变量
if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo "⚠️  请先设置 DEEPSEEK_API_KEY 环境变量"
    echo "   export DEEPSEEK_API_KEY=your_api_key_here"
    echo ""
    echo "或者创建 .env 文件："
    echo "   cp .env.example .env"
    echo "   # 编辑 .env 文件设置你的 API Key"
    echo ""
    exit 1
fi

echo "✅ 环境变量检查通过"
echo ""

echo "🔧 编译项目..."
cargo build --release
echo ""

echo "📊 查看历史记录统计..."
echo "================================="
./target/release/rust-daily --stats
echo ""

echo "📰 生成今日 Rust 技术日报（单条处理模式）..."
echo "================================="
./target/release/rust-daily

echo ""
echo "📁 生成日报文件（单条模式）..."
./target/release/rust-daily --output "rust-daily-single-$(date +%Y-%m-%d).md"

echo ""
echo "🔄 再次运行（演示去重功能）..."
echo "================================="
./target/release/rust-daily

echo ""
echo "⚡ 批量处理模式演示..."
echo "================================="
./target/release/rust-daily --batch-mode --output "rust-daily-batch-$(date +%Y-%m-%d).md"

echo ""
echo "💪 强制模式演示..."
echo "================================="
./target/release/rust-daily --force --output "rust-daily-force-$(date +%Y-%m-%d).md"

echo ""
echo "📈 最终统计信息..."
echo "================================="
./target/release/rust-daily --stats

echo ""
echo "🎉 演示完成！"
echo "功能演示："
echo "   ✅ 单条详细处理模式"
echo "   ✅ 批量快速处理模式"
echo "   ✅ 自动去重处理"
echo "   ✅ 历史记录统计"
echo "   ✅ 强制模式重新处理"
echo "   ✅ 文件输出功能"
echo ""
echo "生成的文件："
echo "   - rust-daily-single-$(date +%Y-%m-%d).md (单条处理版本)"
echo "   - rust-daily-batch-$(date +%Y-%m-%d).md (批量处理版本)"
echo "   - rust-daily-force-$(date +%Y-%m-%d).md (强制版本)"