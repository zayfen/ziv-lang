# 🚀 Lumi 快速开始指南

欢迎使用 Lumi 编程语言！本指南将帮助你在 5 分钟内上手。

---

## 📦 安装

### 方式 1: 从源码构建

```bash
# 1. 克隆仓库
git clone https://github.com/zayfen/lumi.git
cd lumi

# 2. 构建（需要 Rust 1.80+）
cargo build --release

# 3. 编译器位于
./target/release/lumi --version
```

### 方式 2: 下载预编译版本

（即将推出）

---

## 🎯 Hello World

### 1. 创建文件

创建 `hello.lumi`:

```lumi
println("Hello, Lumi! 🌟");
```

### 2. 编译

```bash
./target/release/lumi hello.lumi -o hello
```

### 3. 运行

```bash
./hello
# 输出: Hello, Lumi! 🌟
```

---

## 📚 基础语法

### 变量声明

```lumi
// 整数
let age = 25;

// 浮点数
let pi = 3.14159;

// 字符串
let name = "Lumi";

// 布尔值
let isAwesome = true;
```

### 基本运算

```lumi
let a = 10;
let b = 20;

// 算术运算
let sum = a + b;      // 30
let diff = a - b;     // -10
let product = a * b;  // 200
let quotient = b / a; // 2

// 比较运算
let isEqual = a == b;    // false
let isGreater = a > b;   // false
let isLess = a < b;      // true
```

### 控制流

```lumi
// If-Else
let x = 10;
if (x > 5) {
    println("x 大于 5");
} else {
    println("x 小于等于 5");
}

// While 循环
let i = 0;
while (i < 5) {
    println(i);
    i = i + 1;
}
```

### 函数定义

```lumi
// 简单函数
function greet(name) {
    println("Hello, " + name);
}

greet("World");

// 递归函数
function factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

let result = factorial(5);  // 120
```

---

## 📖 标准库示例

### IO 函数

```lumi
// 打印
print("不换行: ");
println("这会换行");

// 读取输入
print("请输入姓名: ");
let name = read();
println("你好, " + name);
```

### 数学函数

```lumi
// 基本运算
println(abs(-10));      // 10
println(min(5, 10));    // 5
println(max(5, 10));    // 10

// 高级运算
println(sqrt(16));      // 4.0
println(pow(2, 3));     // 8.0

// 取整
println(floor(3.7));    // 3
println(ceil(3.2));     // 4
println(round(3.5));    // 4
```

### 字符串函数

```lumi
let s = "Hello, Lumi!";

// 长度
println(strlen(s));           // 13

// 连接
let greeting = concat("Hello, ", "World");

// 子字符串
let sub = substr(s, 0, 5);    // "Hello"

// 大小写转换
println(to_upper(s));         // "HELLO, LUMI!"
println(to_lower(s));         // "hello, lumi!"

// 去除空白
let trimmed = trim("  Hello  ");

// 包含检查
if (contains(s, "Lumi")) {
    println("找到 Lumi!");
}
```

### 数组函数

```lumi
let arr = [1, 2, 3, 4, 5];

// 长度
println(arrlen(arr));         // 5

// 访问元素
println(first(arr));          // 1
println(last(arr));           // 5
println(get(arr, 2));         // 3

// 修改数组
let newArr = push(arr, 6);    // [1, 2, 3, 4, 5, 6]
let popped = pop(arr);        // 5
let reversed = reverse(arr);  // [5, 4, 3, 2, 1]
```

---

## 🎨 实战示例

### Fibonacci 数列

```lumi
function fib(n) {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

// 打印前 10 个 Fibonacci 数
let i = 0;
while (i < 10) {
    println(fib(i));
    i = i + 1;
}
```

### 计算器

```lumi
function calculate(a, op, b) {
    if (op == "+") {
        return a + b;
    } else if (op == "-") {
        return a - b;
    } else if (op == "*") {
        return a * b;
    } else if (op == "/") {
        return a / b;
    }
    return 0;
}

let result = calculate(10, "+", 20);
println(result);  // 30
```

---

## 🔧 编译器选项

```bash
# 基本用法
lumi source.lumi -o output

# 保留汇编文件
lumi source.lumi -o output --keep-asm

# 查看帮助
lumi --help
```

---

## 📂 项目结构

```
lumi/
├── src/
│   ├── lexer/      - 词法分析
│   ├── parser/     - 语法分析
│   ├── semantic/   - 语义分析
│   ├── ir/         - 中间表示
│   ├── codegen/    - 代码生成
│   ├── stdlib/     - 标准库
│   └── compiler.rs - 编译器驱动
├── examples/       - 示例代码
│   └── stdlib/     - 标准库示例
├── docs/           - 文档
└── tests/          - 测试
```

---

## 🐛 常见问题

### Q: 编译错误怎么办？

A: 检查语法错误，Lumi 会给出详细的错误信息和位置。

### Q: 如何调试？

A: 使用 `--keep-asm` 选项查看生成的汇编代码。

### Q: 支持哪些平台？

A: 目前支持 x86-64 Linux，ARM64 支持正在开发中。

---

## 📚 下一步

- 阅读完整 [API 文档](STDLIB_API.md)
- 浏览 [示例代码](../examples/stdlib/)
- 加入社区讨论

---

**祝你编程愉快！** 🌟
