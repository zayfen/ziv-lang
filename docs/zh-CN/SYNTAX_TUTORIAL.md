# Ziv 语法教程（当前实现）

本文档描述的是当前仓库实现中可用、可编译的语法子集。

## 1. 文件与语句

- 源文件扩展名：`.ziv`
- 语句分隔：分号 `;` 建议保留；当前解析器在多数场景下允许省略
- 注释：`//` 单行注释

示例：

```ziv
// 建议每条语句加分号
let x = 1;
let y = 2;
println(x + y);
```

## 2. 变量声明与赋值

### 2.1 `let` / `const`

```ziv
let a = 10;
const b = 20;
let c: int = 30;
let d;          // 允许无初始化
```

### 2.2 普通赋值

```ziv
let n = 1;
n = n + 1;
```

## 3. 类型注解

当前常见可用注解：

- `int` / `i64`
- `float` / `f64`（语义层支持；代码生成目前以 `i64` 为主）
- `string`
- `bool`
- `any`
- `function`
- 结构体名（如 `Person`）

示例：

```ziv
let name: string = "ziv";
let flag: bool = true;
let f: function;
```

## 4. 表达式

### 4.1 字面量

```ziv
42
3.14
"hello"
true
false
```

### 4.2 运算符

当前支持的二元运算：

- 算术：`+ - * /`
- 比较：`== != < <= > >=`

示例：

```ziv
let x = (1 + 2) * 3;
if (x >= 9) {
    println(x);
}
```

### 4.3 一元负号

```ziv
let x = -10;
```

## 5. 控制流

### 5.1 `if / else`

```ziv
if (x > 0) {
    println("positive");
} else {
    println("non-positive");
}
```

### 5.2 `while`

```ziv
let i = 0;
while (i < 3) {
    println(i);
    i = i + 1;
}
```

## 6. 函数

### 6.1 函数定义

```ziv
function add(a: int, b: int): int {
    return a + b;
}
```

### 6.2 函数调用

```ziv
let r = add(1, 2);
println(r);
```

### 6.3 函数作为参数

```ziv
function inc(x: int): int {
    return x + 1;
}

function apply(f: function, v: int): int {
    return f(v);
}

println(apply(inc, 41));
```

## 7. 结构体 `struct`

### 7.1 定义

```ziv
struct Person {
    age: int;
    score: int;
}
```

### 7.2 构造

使用 `StructName.(...)` 语法：

```ziv
let p: Person = Person.(age = 18, score = 90);
```

### 7.3 字段访问

```ziv
println(p.age);
```

### 7.4 字段覆盖合并（`+=`）

```ziv
p += Person.(age = 20);
println(p.age);   // 20
println(p.score); // 90
```

## 8. 模块导入

当前支持：

```ziv
from { "./math.ziv" } import { add, sub };
```

也支持不带路径花括号的写法：

```ziv
from "./math.ziv" import { add };
```

说明：

- 导入的是目标文件的顶层符号（函数、变量、结构体名）。
- 编译器会递归解析并合并导入模块。

## 9. 内置函数调用

标准库函数可直接调用，例如：

```ziv
println("hello");
abs(-10);
parseInt("123", 10);
```

运行时说明：

- 当前默认可执行运行时可执行全部标准库函数（117 个）。
- `net` / `crypto` / `utils` 的部分函数目前使用轻量实现，重点保证接口稳定与可测试性；需要完整生产语义时建议接入外部宿主 runtime。

## 10. 示例索引

- 标准库示例：`examples/stdlib/`
- 结构体示例：`examples/struct/struct_demo.ziv`
- 结构体参数/返回示例：`examples/struct/struct_func_demo.ziv`
- 函数参数传函数示例：`examples/function/function_arg_demo.ziv`
- 导入示例：`examples/from_import/full_demo.ziv`

## 11. 当前不在本文范围的语法

以下能力尚未作为“稳定语法教程”收录：

- 数组字面量与索引表达式语法
- `for`、`switch`、`class` 等高级语法
- 完整函数类型签名（如 `function(int)->int`）

建议以仓库测试与 examples 为准逐步扩展。
