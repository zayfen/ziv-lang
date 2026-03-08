# Ziv 中文文档

## 文档列表

- 语法教程：[SYNTAX_TUTORIAL.md](SYNTAX_TUTORIAL.md)
- 标准库使用指南：[STDLIB_GUIDE.md](STDLIB_GUIDE.md)
- 标准库完整 API：[STDLIB_API.md](STDLIB_API.md)
- 编译器架构（英文）：[../ARCHITECTURE.md](../ARCHITECTURE.md)

## 推荐阅读顺序

1. 先看语法教程，熟悉当前语法边界与可用特性。
2. 再看标准库使用指南，理解标准库注册、链接、运行时行为。
3. 查阅标准库 API 表，按函数名检索签名。

## 快速验证

```bash
cargo test --workspace --all-targets
./target/debug/ziv examples/stdlib/hello.ziv -o /tmp/hello && /tmp/hello
for f in examples/stdlib/*.ziv; do
  ./target/debug/ziv "$f" -o /tmp/ziv_example && /tmp/ziv_example </dev/null
done
```
