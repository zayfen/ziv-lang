# Lumi 🌟

**A modern systems programming language that compiles to native ELF executables**

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

---

## 📖 简介

**Lumi**（拉丁语 "光"）是一个现代化的系统编程语言，设计目标是：

- ✨ **简洁优雅** - 类 JavaScript 语法，易于学习
- 🚀 **原生性能** - 直接编译为 x86-64 ELF 可执行文件
- 🛡️ **类型安全** - 强类型系统，编译时检查
- 🔧 **现代工具链** - 基于 Rust 实现，可靠且高效

---

## 🎯 特性

### 核心功能

- ✅ **完整的编译流水线**
  - Lexer（词法分析）
  - Parser（语法分析）
  - Semantic Analyzer（语义分析）
  - IR Generation（中间表示）
  - Code Generation（代码生成）
  - ELF Linking（链接）

- ✅ **语言特性**
  - 变量声明（`let`）
  - 基本类型（整数、浮点数、字符串、布尔值）
  - 算术运算
  - 控制流（if/else, while）
  - 函数定义和调用
  - 递归支持

- ✅ **标准库**
  - IO 函数：print, println, read
  - 数学函数：abs, min, max, sqrt, pow
  - 字符串函数：strlen, concat, substr, trim
  - 数组函数：push, pop, arrlen, reverse

- ✅ **多平台代码生成**
  - x86-64 汇编（GNU as）
  - ARM64 支持
  - Cranelift JIT 后端

---

## 🚀 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/zayfen/lumi.git
cd lumi

# 构建
cargo build --release

# 编译器位于 target/release/lumi
```

### Hello World

创建 `hello.lumi` 文件：

```lumi
println("Hello, Lumi! 🌟");
```

编译并运行：

```bash
./target/release/lumi hello.lumi -o hello
./hello
# 输出: Hello, Lumi! 🌟
```

---

## 📚 示例代码

### 1. 基本运算

```lumi
let a = 10;
let b = 20;
let c = a + b;
println(c);  // 输出: 30
```

### 2. Fibonacci 数列

```lumi
function fib(n) {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

let result = fib(10);
println(result);  // 输出: 55
```

### 3. 标准库使用

```lumi
// 字符串操作
let s = "Hello, World!";
println(strlen(s));           // 13
println(to_upper(s));          // HELLO, WORLD!

// 数学函数
println(abs(-10));            // 10
println(max(5, 10));          // 10
println(sqrt(16));            // 4.0

// 数组操作
let arr = [1, 2, 3, 4, 5];
println(arrlen(arr));         // 5
println(first(arr));          // 1
println(last(arr));           // 5
```

更多示例见 [examples/stdlib/](examples/stdlib/)

---

## 📖 文档

- [标准库 API 文档](docs/STDLIB_API.md)
- [开发文档](CLAUDE.md)
- [架构设计](docs/ARCHITECTURE.md)

---

## 🛠️ 编译器选项

```bash
lumi <source.lumi> [options]

选项:
  -o <output>      输出文件名（默认: a.out）
  --keep-asm       保留生成的汇编文件
  --help           显示帮助信息
```

---

## 📊 项目状态

| 模块 | 状态 | 代码行数 |
|------|------|---------|
| Lexer | ✅ 完成 | 346 |
| Parser | ✅ 完成 | 333 |
| Semantic | ✅ 完成 | 485 |
| IR | ✅ 完成 | 296 |
| CodeGen | ✅ 完成 | 220 |
| 标准库 | ✅ 完成 | 689 |
| **总计** | - | **2,369** |

---

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行标准库测试
cargo test --lib stdlib

# 测试覆盖率
cargo tarpaulin --out Html
```

---

## 🤝 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md)

---

## 📝 许可证

双重许可：

- MIT License
- Apache License 2.0

详见 [LICENSE-MIT](LICENSE-MIT) 和 [LICENSE-APACHE](LICENSE-APACHE)

---

## 🙏 致谢

感谢以下开源项目：

- [Rust](https://www.rust-lang.org/) - 编程语言
- [Logos](https://github.com/maciejhirsz/logos) - Lexer 生成器
- [LALRPOP](https://github.com/lalrpop/lalrpop) - Parser 生成器
- [Cranelift](https://github.com/bytecodealliance/wasmtime/tree/main/cranelift) - 代码生成后端

---

## 📮 联系方式

- **作者**: Zayfen
- **GitHub**: https://github.com/zayfen/lumi
- **Issues**: https://github.com/zayfen/lumi/issues

---

**Lumi** - 让编程更简单、更高效！ 🌟
