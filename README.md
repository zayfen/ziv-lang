# Ziv

Ziv 是一门正在快速演进的系统编程语言，语法风格接近 JavaScript，编译目标是原生可执行文件。

## 文档导航

- 中文文档入口：[docs/zh-CN/README.md](docs/zh-CN/README.md)
- 语法教程（中文）：[docs/zh-CN/SYNTAX_TUTORIAL.md](docs/zh-CN/SYNTAX_TUTORIAL.md)
- 标准库使用指南（中文）：[docs/zh-CN/STDLIB_GUIDE.md](docs/zh-CN/STDLIB_GUIDE.md)
- 标准库完整 API（中文）：[docs/zh-CN/STDLIB_API.md](docs/zh-CN/STDLIB_API.md)
- 编译器架构：[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

## 当前能力概览

- 完整编译流水线：Lexer -> Parser -> Semantic -> IR -> Codegen -> Link
- 语言特性：
  - `from {"path"} import { symbol }` 模块导入
  - `let/const` 声明、赋值
  - `function` 定义、返回值、函数作为参数
  - `if/else`、`while`、块作用域
  - `struct` 定义、构造、字段访问、`+=` 字段覆盖
- 标准库注册：117 个内置函数（IO/Math/String/Array/Container/Utils/Filesystem/Net/Crypto/Encoding）
- 代码生成后端：Cranelift（默认）、x86_64、ARM64

## 重要说明

- 当前默认可执行运行时已内置并可调用全部 10 类标准库函数（共 117 个）。
- 这些函数已覆盖语义注册、IR 降级、链接与 examples/测试验证链路。
- `net` / `crypto` / `utils` 中部分能力当前采用轻量实现，目标是提供稳定接口与可重复测试结果；若需要完整生产级协议栈与安全语义，建议通过外部运行时对象或宿主库替换。

## 快速开始

### 构建

```bash
cargo build --release
```

### 编译并运行

```bash
./target/release/ziv examples/stdlib/hello.ziv -o hello
./hello
```

### 运行测试

```bash
cargo test --workspace --all-targets
```

### 运行标准库示例（批量）

```bash
for f in examples/stdlib/*.ziv; do
  ./target/debug/ziv "$f" -o /tmp/ziv_example && /tmp/ziv_example </dev/null
done
```

## 示例目录

- 标准库调用示例：`examples/stdlib/`
- `struct` 示例：`examples/struct/`
- `from ... import` 示例：`examples/from_import/`
- 函数参数传函数示例：`examples/function/function_arg_demo.ziv`

## 命令行

```bash
ziv <source.ziv> [options]

options:
  -o <output>     指定输出可执行文件名（默认 a.out）
  --keep-asm      保留中间汇编/目标文件
```
