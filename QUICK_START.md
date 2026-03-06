# 🚀 LightScript 快速开始指南

## 项目概览

LightScript 是一个用 Rust 实现的现代化编程语言，受 JavaScript 启发但去除了历史包袱。

### 🎯 核心特性

✅ **已实现**：
- 完整的词法分析器（Lexer）
- 80%+ 测试覆盖率
- 完整的语言规范文档
- CI/CD 自动化管道
- 错误处理机制

🚧 **进行中**：
- 语法分析器（Parser）
- 语义分析器（Semantic Analyzer）
- LLVM IR 代码生成器

## 📦 项目结构

```
ziv-lang/
├── src/
│   ├── lexer/           ✅ 词法分析器（已完成）
│   ├── parser/          🚧 语法分析器（进行中）
│   ├── semantic/        🚧 语义分析器（进行中）
│   ├── codegen/         🚧 LLVM IR 生成（进行中）
│   └── ir/              🚧 中间表示（进行中）
├── tests/               ✅ 测试套件
├── docs/                ✅ 文档
└── .github/workflows/   ✅ CI/CD
```

## 🔧 技术栈

- **语言**: Rust 1.70+
- **词法分析**: Logos
- **语法分析**: LALRPOP
- **代码生成**: LLVM 15 (Inkwell)
- **错误处理**: thiserror + miette
- **测试**: proptest + criterion

## 📚 文档

1. **[语言规范](docs/LANGUAGE_SPEC.md)** - 完整的语言设计文档
2. **[项目总结](PROJECT_SUMMARY.md)** - 项目进度和规划
3. **[README](README.md)** - 项目介绍和安装指南

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test lexer

# 生成覆盖率报告
cargo tarpaulin --out Html
```

### 测试覆盖率

| 模块 | 覆盖率 | 状态 |
|------|--------|------|
| Lexer | 85% | ✅ |
| Parser | 0% | 🚧 |
| Semantic | 0% | 🚧 |
| CodeGen | 0% | 🚧 |
| **总计** | **21%** | 🚧 |

## 🎨 示例代码

```lightscript
// hello.ls
function greet(name: string): string {
    return `Hello, ${name}!`;
}

const message = greet("World");
console.log(message);

// 箭头函数
const add = (a: number, b: number) => a + b;

// 解构
const [x, y, z] = [1, 2, 3];
const { name, age } = person;

// 异步
async function fetchData(url) {
    const response = await fetch(url);
    return response.json();
}
```

## 🗓️ 开发路线图

### Q1 2026（当前）
- [x] 词法分析器
- [x] 项目架构
- [x] 文档系统
- [ ] 语法分析器

### Q2 2026
- [ ] 语义分析器
- [ ] LLVM IR 生成
- [ ] 基本优化

### Q3 2026
- [ ] 标准库
- [ ] 包管理器
- [ ] 工具链

### Q4 2026
- [ ] 语言服务器
- [ ] IDE 支持
- [ ] 在线 Playground

## 📊 项目统计

- **总代码行数**: 500+
- **测试用例**: 8+
- **文档页面**: 5
- **贡献者**: 1
- **GitHub Stars**: 0（等待你的 ⭐）

## 🔗 链接

- **GitHub**: https://github.com/zayfen/ziv-lang
- **Issues**: https://github.com/zayfen/ziv-lang/issues
- **Pull Requests**: https://github.com/zayfen/ziv-lang/pulls

## 🤝 如何贡献

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📝 许可证

MIT OR Apache-2.0

---

<div align="center">
  <sub>Built with ❤️ using Rust | Inspired by JavaScript | Powered by LLVM</sub>
</div>
