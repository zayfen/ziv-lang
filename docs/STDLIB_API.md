# Lumi 标准库 API 文档

**版本**: 0.1.0  
**最后更新**: 2026-03-07

---

## 📚 概述

Lumi 标准库提供了常用的内置函数，包括 IO 操作、数学计算、字符串处理和数组操作。

---

## 📦 模块结构

```
stdlib/
├── io.rs      - 输入输出函数
├── math.rs    - 数学函数
├── string.rs  - 字符串处理函数
└── array.rs   - 数组操作函数
```

---

## 🔧 IO 函数

### `print(value: any) -> void`

**功能**: 打印值到标准输出，不换行

**参数**:
- `value`: 要打印的值（可以是任何类型）

**返回值**: 无

**示例**:
```lumi
print("Hello");
print(" ");
print("World");
// 输出: Hello World
```

---

### `println(value: any) -> void`

**功能**: 打印值到标准输出，并换行

**参数**:
- `value`: 要打印的值

**返回值**: 无

**示例**:
```lumi
println("Hello, Lumi!");
println(42);
```

---

### `read() -> string`

**功能**: 从标准输入读取一行

**参数**: 无

**返回值**: 读取的字符串

**示例**:
```lumi
print("请输入姓名: ");
let name = read();
println(name);
```

---

### `eprint(value: any) -> void`

**功能**: 打印值到标准错误，不换行

**参数**:
- `value`: 要打印的值

**返回值**: 无

---

### `eprintln(value: any) -> void`

**功能**: 打印值到标准错误，并换行

**参数**:
- `value`: 要打印的值

**返回值**: 无

---

## 🔢 数学函数

### `abs(x: number) -> number`

**功能**: 返回数字的绝对值

**参数**:
- `x`: 数字

**返回值**: 绝对值

**示例**:
```lumi
let result = abs(-10);  // 10
```

---

### `min(a: number, b: number) -> number`

**功能**: 返回两个数中的最小值

**参数**:
- `a`: 第一个数字
- `b`: 第二个数字

**返回值**: 较小的数字

**示例**:
```lumi
let result = min(5, 10);  // 5
```

---

### `max(a: number, b: number) -> number`

**功能**: 返回两个数中的最大值

**参数**:
- `a`: 第一个数字
- `b`: 第二个数字

**返回值**: 较大的数字

**示例**:
```lumi
let result = max(5, 10);  // 10
```

---

### `sqrt(x: number) -> f64`

**功能**: 返回数字的平方根

**参数**:
- `x`: 数字

**返回值**: 平方根（浮点数）

**示例**:
```lumi
let result = sqrt(16);  // 4.0
```

---

### `pow(base: number, exp: number) -> f64`

**功能**: 返回 base 的 exp 次幂

**参数**:
- `base`: 底数
- `exp`: 指数

**返回值**: 幂运算结果（浮点数）

**示例**:
```lumi
let result = pow(2, 3);  // 8.0
```

---

### `floor(x: number) -> i64`

**功能**: 向下取整

**参数**:
- `x`: 数字

**返回值**: 整数

**示例**:
```lumi
let result = floor(3.7);  // 3
```

---

### `ceil(x: number) -> i64`

**功能**: 向上取整

**参数**:
- `x`: 数字

**返回值**: 整数

**示例**:
```lumi
let result = ceil(3.2);  // 4
```

---

### `round(x: number) -> i64`

**功能**: 四舍五入

**参数**:
- `x`: 数字

**返回值**: 整数

**示例**:
```lumi
let result = round(3.5);  // 4
```

---

## 📝 字符串函数

### `strlen(s: string) -> i64`

**功能**: 返回字符串长度

**参数**:
- `s`: 字符串

**返回值**: 长度

**示例**:
```lumi
let len = strlen("Hello");  // 5
```

---

### `concat(a: string, b: string) -> string`

**功能**: 连接两个字符串

**参数**:
- `a`: 第一个字符串
- `b`: 第二个字符串

**返回值**: 连接后的字符串

**示例**:
```lumi
let result = concat("Hello", "World");  // "HelloWorld"
```

---

### `substr(s: string, start: i64, length: i64) -> string`

**功能**: 获取子字符串

**参数**:
- `s`: 原字符串
- `start`: 起始位置
- `length`: 长度

**返回值**: 子字符串

**示例**:
```lumi
let result = substr("Hello, World!", 0, 5);  // "Hello"
```

---

### `char_at(s: string, index: i64) -> char`

**功能**: 获取指定位置的字符

**参数**:
- `s`: 字符串
- `index`: 索引

**返回值**: 字符

**示例**:
```lumi
let ch = char_at("Hello", 0);  // 'H'
```

---

### `to_upper(s: string) -> string`

**功能**: 转换为大写

**参数**:
- `s`: 字符串

**返回值**: 大写字符串

**示例**:
```lumi
let result = to_upper("hello");  // "HELLO"
```

---

### `to_lower(s: string) -> string`

**功能**: 转换为小写

**参数**:
- `s`: 字符串

**返回值**: 小写字符串

**示例**:
```lumi
let result = to_lower("HELLO");  // "hello"
```

---

### `trim(s: string) -> string`

**功能**: 去除首尾空白

**参数**:
- `s`: 字符串

**返回值**: 去除空白后的字符串

**示例**:
```lumi
let result = trim("  Hello  ");  // "Hello"
```

---

### `contains(s: string, substr: string) -> bool`

**功能**: 检查是否包含子字符串

**参数**:
- `s`: 原字符串
- `substr`: 子字符串

**返回值**: 布尔值

**示例**:
```lumi
let found = contains("Hello, World!", "World");  // true
```

---

## 📚 数组函数

### `push(arr: array, element: any) -> array`

**功能**: 在数组末尾添加元素

**参数**:
- `arr`: 数组
- `element`: 要添加的元素

**返回值**: 新数组

**示例**:
```lumi
let arr = [1, 2, 3];
let newArr = push(arr, 4);  // [1, 2, 3, 4]
```

---

### `pop(arr: array) -> any`

**功能**: 移除并返回数组末尾元素

**参数**:
- `arr`: 数组

**返回值**: 移除的元素

**示例**:
```lumi
let arr = [1, 2, 3];
let last = pop(arr);  // 3
```

---

### `arrlen(arr: array) -> i64`

**功能**: 返回数组长度

**参数**:
- `arr`: 数组

**返回值**: 长度

**示例**:
```lumi
let arr = [1, 2, 3, 4, 5];
let len = arrlen(arr);  // 5
```

---

### `get(arr: array, index: i64) -> any`

**功能**: 获取指定位置的元素

**参数**:
- `arr`: 数组
- `index`: 索引

**返回值**: 元素

**示例**:
```lumi
let arr = [10, 20, 30];
let elem = get(arr, 1);  // 20
```

---

### `set(arr: array, index: i64, value: any) -> array`

**功能**: 设置指定位置的元素

**参数**:
- `arr`: 数组
- `index`: 索引
- `value`: 新值

**返回值**: 新数组

**示例**:
```lumi
let arr = [1, 2, 3];
let newArr = set(arr, 1, 20);  // [1, 20, 3]
```

---

### `first(arr: array) -> any`

**功能**: 获取第一个元素

**参数**:
- `arr`: 数组

**返回值**: 第一个元素

**示例**:
```lumi
let arr = [1, 2, 3];
let first = first(arr);  // 1
```

---

### `last(arr: array) -> any`

**功能**: 获取最后一个元素

**参数**:
- `arr`: 数组

**返回值**: 最后一个元素

**示例**:
```lumi
let arr = [1, 2, 3];
let last = last(arr);  // 3
```

---

### `reverse(arr: array) -> array`

**功能**: 反转数组

**参数**:
- `arr`: 数组

**返回值**: 反转后的数组

**示例**:
```lumi
let arr = [1, 2, 3];
let reversed = reverse(arr);  // [3, 2, 1]
```

---

## 📊 函数总览

| 类别 | 函数数量 | 主要功能 |
|------|---------|---------|
| IO | 5 | 输入输出 |
| 数学 | 8 | 数学计算 |
| 字符串 | 8 | 字符串处理 |
| 数组 | 8 | 数组操作 |
| **总计** | **29** | - |

---

## 🔗 相关链接

- [Lumi 语言介绍](../README.md)
- [示例代码](../examples/stdlib/)
- [开发文档](../CLAUDE.md)

---

**文档版本**: 1.0  
**维护者**: Zayfen
