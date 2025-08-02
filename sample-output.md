【Rust日报】2025-08-02 

This Week in Rust #610
----------------------

阅读：[https://this-week-in-rust.org/blog/2025/07/30/this-week-in-rust-610/](https://this-week-in-rust.org/blog/2025/07/30/this-week-in-rust-610/)

文章《构建一个简单的哈希图》
--------------

这篇文章介绍了如何用 Rust 语言从头构建一个简单的哈希表。哈希表是一种高效的数据结构，能够在平均 O(1) 的时间复杂度内完成插入和查询操作。

文章首先解释了哈希函数的作用，即通过将键映射为一个数字（哈希值）来快速定位键值对在内部存储中的位置。接着，文章探讨了如何通过"桶"（buckets）来解决哈希冲突问题，即当不同键产生相同哈希值时的情况。

作者还介绍了如何通过动态调整桶的数量来保持哈希表的性能。文章最后提供了实现哈希表的 Rust 代码，包括插入、查询和扩容等关键功能，并通过测试验证了其正确性。

尽管这个哈希表实现相对简单，但它为理解哈希表的工作原理提供了一个很好的起点。

[Reddit](https://www.reddit.com/r/rust/comments/1membxx/my_first_blog_post_building_a_simple_hash_map/) | 阅读：[https://viniciusx.com/blog/building-a-hash-map/](https://viniciusx.com/blog/building-a-hash-map/)

Eon：简单且友好的配置格式
--------------

Eon 是一种简单易用的配置文件格式，旨在替代 Toml 和 YAML。

它使用 `.eon` 文件扩展名，语法类似于 JSON，但更简洁，支持任意类型的键值对和注释。Eon 提供了强大的功能，如支持特殊浮点数（`+inf`、`-inf`、`+nan`）和命名的枚举变体。

主要特点：
* 简洁易读的语法
* 支持注释
* 支持任意类型的键值对
* 与 Serde 集成良好
* 包含 `eonfmt` 格式化工具

使用示例：
```
// Comment
string: "Hello Eon!"
list: [1, 2, 3]
map: {
    boolean: true
    literal_string: 'Can contain \ and "quotes"'
}
```

[Reddit](https://www.reddit.com/r/rust/comments/1mesquw/eon_a_humanfriendly_replacement_for_toml_and_yaml/) | 仓库：[https://github.com/emilk/eon](https://github.com/emilk/eon)

cargo-license：查看依赖的 license
---------------------------

一个实用的 Cargo 子命令，用于检查项目依赖的许可证信息。该工具可以帮助开发者快速了解项目中所有依赖包的许可证类型，对于开源项目和商业项目的许可证合规性检查非常有用。

仓库：[https://github.com/onur/cargo-license](https://github.com/onur/cargo-license)

讨论：serde_yaml 的替代
-----------------

社区讨论 `serde_yaml` 被弃用后的替代方案选择。

"我用 serde_yaml 没问题。不更新又不是坏事。"

"别用 **serde_yml**，那是低质量 AI 生成的库。"

Reddit：
* [https://www.reddit.com/r/rust/comments/1mbo9dl/alternative_for_serde_yaml/](https://www.reddit.com/r/rust/comments/1mbo9dl/alternative_for_serde_yaml/)

--

From 日报小组 Rust Daily

*Generated at 2025-08-02 10:30:00 UTC by Rust Daily*