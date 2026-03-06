# LightLang 语言规范 v0.1

## 🎯 设计理念

LightLang 是一门**现代系统级编程语言**，核心理念：

> **JavaScript 的优雅 + C 的性能 + 现代内存安全**

### 为什么选择 LightLang？

```ziv
// 像 JavaScript 一样优雅
let greet = (name) => `Hello, ${name}!`;

// 像系统语言一样强大
fn main(): i32 {
    let msg = "Hello from LightLang!\n";
    syscall::write(1, msg.as_bytes());  // 直接系统调用
    return 0;
}

// 内存安全，但不需要 GC
let data = [1, 2, 3];
let first = data[0];  // 自动内存管理
// data 离开作用域自动释放
```

---

## 📖 目录

1. [词法结构](#词法结构)
2. [类型系统](#类型系统)
3. [变量与作用域](#变量与作用域)
4. [表达式](#表达式)
5. [语句](#语句)
6. [函数](#函数)
7. [类与对象](#类与对象)
8. [内存管理](#内存管理)
9. [系统编程](#系统编程)
10. [模块系统](#模块系统)

---

## 词法结构

### 关键字

```ziv
// 变量声明
let const

// 函数
function fn return

// 类
class extends constructor

// 控制流
if else switch case default
while for do break continue

// 类型
type interface enum

// 修饰符
static async await

// 模块
import export from

// 特殊
null undefined true false this new
```

### 标识符

```text
identifier = [a-zA-Z_$][a-zA-Z0-9_$]*
```

**示例**：
```ziv
foo
_bar
$element
MyClass
myFunction
```

### 字面量

#### 数字
```ziv
42              // 整数（默认 32 位）
127i8           // 8 位整数
255u8           // 8 位无符号
1024i64         // 64 位整数
3.14            // 浮点数（默认 64 位）
2.5f32          // 32 位浮点
0xFF            // 十六进制
0b1010          // 二进制
0o755           // 八进制
1_000_000       // 数字分隔符
```

#### 字符串
```ziv
"hello"                     // 双引号字符串
'world'                     // 单引号字符串（同双引号）
`hello ${name}`             // 模板字符串
`多行
   字符串`                  // 多行字符串
```

#### 布尔与空值
```ziv
true
false
null          // 空值
undefined     // 未定义
```

### 注释

```ziv
// 单行注释

/*
  多行注释
*/

/**
 * 文档注释
 * @param x - 参数说明
 */
```

---

## 类型系统

### 基本类型

| 类型 | 描述 | 示例 |
|------|------|------|
| `int` | 整数（32位） | `42` |
| `i8`, `i16`, `i32`, `i64` | 固定大小整数 | `127i8` |
| `uint` | 无符号整数 | `255u` |
| `u8`, `u16`, `u32`, `u64` | 固定大小无符号 | `255u8` |
| `float` | 浮点数（64位） | `3.14` |
| `f32`, `f64` | 固定大小浮点 | `2.5f32` |
| `bool` | 布尔值 | `true` |
| `char` | Unicode 字符 | `'字'` |
| `string` | 字符串 | `"hello"` |

### 类型推断

```ziv
// 自动推断
let x = 42;           // int
let y = 3.14;         // float
let s = "hello";      // string
let arr = [1, 2, 3];  // int[]

// 显式注解
let x: i64 = 42;
let arr: float[] = [1.0, 2.0, 3.0];
```

### 复合类型

#### 数组
```ziv
// 固定大小
let arr: int[5] = [1, 2, 3, 4, 5];

// 动态数组
let vec: int[] = [1, 2, 3];
vec.push(4);          // [1, 2, 3, 4]

// 多维
let matrix: float[][] = [[1.0, 2.0], [3.0, 4.0]];
```

#### 对象
```ziv
// 对象字面量
let person = {
    name: "Alice",
    age: 30,
};

// 类型注解
type Person = {
    name: string,
    age: int,
};

let p: Person = {
    name: "Alice",
    age: 30,
};
```

#### 元组
```ziv
let tuple: (int, string, float) = (1, "hello", 3.14);
let (x, y, z) = tuple;  // 解构
```

#### 枚举
```ziv
enum Status {
    Pending,
    Active,
    Completed,
}

let status = Status::Active;

// 带数据的枚举
enum Option<T> {
    Some(T),
    None,
}

let opt = Option::Some(42);
```

#### 联合类型
```ziv
type StringOrNumber = string | int;

let value: StringOrNumber = "hello";
value = 42;  // OK
```

#### 可选类型
```ziv
let name: string? = null;  // 可以是 string 或 null

// 等价于
let name: string | null = null;
```

### 泛型

```ziv
// 泛型函数
function identity<T>(x: T): T {
    return x;
}

let num = identity(42);        // T = int
let str = identity("hello");   // T = string

// 泛型类
class Container<T> {
    private value: T;
    
    constructor(value: T) {
        this.value = value;
    }
    
    get(): T {
        return this.value;
    }
}

let container = new Container(42);
```

---

## 变量与作用域

### 变量声明

```ziv
// let: 可变变量
let x = 10;
x = 20;  // OK

// const: 不可变变量
const PI = 3.14159;
// PI = 3.14;  // ❌ 错误
```

### 作用域

```ziv
// 块级作用域
{
    let x = 10;
    console.log(x);  // 10
}
// console.log(x);  // ❌ 错误：x 不存在

// 函数作用域
function example() {
    let y = 20;
    console.log(y);  // 20
}
```

### 变量提升

```ziv
// ❌ 不支持变量提升（与 JavaScript 不同）
console.log(x);  // 错误：x 未定义
let x = 10;
```

---

## 表达式

### 算术运算

```ziv
1 + 2       // 3
3 - 1       // 2
2 * 3       // 6
10 / 2      // 5
7 % 3       // 1
2 ** 3      // 8
```

### 比较运算

```ziv
1 == 1      // true（值比较）
1 != 2      // true
1 === 1     // true（严格相等）
1 !== "1"   // true
1 < 2       // true
1 <= 2      // true
1 > 0       // true
1 >= 0      // true
```

### 逻辑运算

```ziv
true && false   // false
true || false   // true
!true           // false
```

### 位运算

```ziv
0b1010 & 0b1100   // 0b1000
0b1010 | 0b1100   // 0b1110
0b1010 ^ 0b1100   // 0b0110
~0b1010           // 0b0101
0b1010 << 2       // 0b101000
0b1010 >> 2       // 0b10
```

### 解构

```ziv
// 数组解构
let [a, b, c] = [1, 2, 3];
let [first, ...rest] = [1, 2, 3, 4];

// 对象解构
let { name, age } = person;
let { x: px, y: py } = point;

// 默认值
let { x = 0, y = 0 } = point;
```

### 展开运算符

```ziv
// 数组展开
let arr1 = [1, 2, 3];
let arr2 = [...arr1, 4, 5];  // [1, 2, 3, 4, 5]

// 对象展开
let obj1 = { a: 1 };
let obj2 = { ...obj1, b: 2 };  // { a: 1, b: 2 }
```

### 可选链与空值合并

```ziv
let city = person?.address?.city;  // 安全访问
let len = arr?.length ?? 0;        // 空值合并
```

---

## 语句

### 条件语句

```ziv
// if-else
if (x > 0) {
    console.log("positive");
} else if (x < 0) {
    console.log("negative");
} else {
    console.log("zero");
}

// 三元运算符
let result = x > 0 ? "positive" : "non-positive";

// switch
switch (value) {
    case 1:
        console.log("one");
        break;
    case 2:
    case 3:
        console.log("two or three");
        break;
    default:
        console.log("other");
}
```

### 循环语句

```ziv
// while
let i = 0;
while (i < 10) {
    console.log(i);
    i++;
}

// for
for (let i = 0; i < 10; i++) {
    console.log(i);
}

// for-of
for (const item of array) {
    console.log(item);
}

// for-in
for (const key in object) {
    console.log(key, object[key]);
}
```

### 跳转语句

```ziv
break;          // 跳出循环
continue;       // 继续下一次循环
return value;   // 返回
throw error;    // 抛出异常
```

### 异常处理

```ziv
try {
    throw new Error("something went wrong");
} catch (e) {
    console.error(e.message);
} finally {
    console.log("cleanup");
}
```

---

## 函数

### 函数声明

```ziv
// 普通函数
function add(a: int, b: int): int {
    return a + b;
}

// 箭头函数
const multiply = (a: int, b: int): int => a * b;

// 函数表达式
const divide = function(a: int, b: int): int {
    return a / b;
};
```

### 默认参数

```ziv
function greet(name: string, greeting: string = "Hello") {
    console.log(`${greeting}, ${name}!`);
}

greet("Alice");          // "Hello, Alice!"
greet("Bob", "Hi");      // "Hi, Bob!"
```

### 剩余参数

```ziv
function sum(...numbers: int[]): int {
    return numbers.reduce((a, b) => a + b, 0);
}

sum(1, 2, 3, 4, 5);  // 15
```

### 高阶函数

```ziv
// 函数作为参数
function map<T, U>(arr: T[], fn: (x: T) => U): U[] {
    let result: U[] = [];
    for (const item of arr) {
        result.push(fn(item));
    }
    return result;
}

// 函数作为返回值
function multiplier(factor: int): (int) -> int {
    return (x) => x * factor;
}

let double = multiplier(2);
double(5);  // 10
```

### 闭包

```ziv
function counter() {
    let count = 0;
    return () => {
        count++;
        return count;
    };
}

let c = counter();
c();  // 1
c();  // 2
```

### 异步函数

```ziv
async function fetchData(url: string): Promise<Response> {
    const response = await fetch(url);
    return response.json();
}

// 使用
let data = await fetchData("https://api.example.com/data");
```

---

## 类与对象

### 类定义

```ziv
class Person {
    // 属性
    name: string;
    age: int;
    
    // 构造函数
    constructor(name: string, age: int) {
        this.name = name;
        this.age = age;
    }
    
    // 方法
    greet(): void {
        console.log(`Hello, I'm ${this.name}`);
    }
    
    // 静态方法
    static create(name: string, age: int): Person {
        return new Person(name, age);
    }
}

let person = new Person("Alice", 30);
person.greet();  // "Hello, I'm Alice"
```

### 继承

```ziv
class Student extends Person {
    grade: string;
    
    constructor(name: string, age: int, grade: string) {
        super(name, age);
        this.grade = grade;
    }
    
    study(): void {
        console.log(`${this.name} is studying`);
    }
}

let student = new Student("Bob", 20, "A");
student.greet();  // 继承自 Person
student.study();
```

### 访问修饰符

```ziv
class Example {
    public x: int;           // 公有（默认）
    private y: int;          // 私有
    protected z: int;        // 保护
    readonly id: int;        // 只读
    static count: int = 0;   // 静态
}
```

### 接口

```ziv
interface Drawable {
    draw(): void;
}

class Circle implements Drawable {
    draw(): void {
        console.log("Drawing circle");
    }
}
```

---

## 内存管理

### 自动内存管理

LightLang 使用**区域推断（Region Inference）**进行内存管理：

```ziv
function example() {
    let arr = [1, 2, 3, 4, 5];  // 在函数作用域分配
    // 使用 arr
    console.log(arr[0]);
}  // arr 自动释放
```

### 生命周期规则

1. **作用域绑定**：变量在定义的作用域结束时释放
2. **移动语义**：赋值时转移所有权
3. **引用计数**：共享数据使用引用计数

```ziv
let s1 = "hello";
let s2 = s1;  // s1 移动到 s2
// console.log(s1);  // ❌ 错误：s1 已失效
console.log(s2);     // ✅ OK

// 共享数据
let shared = share(s2);  // 引用计数 +1
console.log(s2);         // ✅ s2 仍然有效
console.log(shared);     // ✅ shared 也有效
```

### 手动内存管理（高级）

```ziv
// 使用 native 关键字标记手动管理
native function malloc(size: uint): *void;
native function free(ptr: *void): void;

function manualExample() {
    let ptr = malloc(100);
    // 使用 ptr
    free(ptr);  // 手动释放
}
```

---

## 系统编程

### 系统调用

```ziv
import syscall from "std/os";

fn main(): i32 {
    let msg = "Hello, World!\n";
    syscall.write(1, msg.as_bytes());  // 写到 stdout
    return 0;
}
```

### 指针操作

```ziv
// 原始指针（需要 unsafe）
unsafe fn pointerExample() {
    let x: int = 10;
    let ptr: *int = &x;
    
    // 解引用
    let value = *ptr;
    
    // 指针运算
    let arr = [1, 2, 3, 4, 5];
    let p: *int = &arr[0];
    let second = *(p + 1);  // 2
}
```

### 内联汇编

```ziv
fn getCPUID(): i32 {
    let eax: i32;
    unsafe {
        asm!(
            "cpuid",
            out("eax") eax,
        );
    }
    return eax;
}
```

### ELF 输出

```bash
$ llc hello.ll -o hello
$ file hello
hello: ELF 64-bit LSB executable, x86-64

$ ./hello
Hello, World!
```

---

## 模块系统

### 导出

```ziv
// math.ll
export function add(a: int, b: int): int {
    return a + b;
}

export const PI = 3.14159;

export class Calculator {
    // ...
}
```

### 导入

```ziv
// 命名导入
import { add, PI } from "./math.ll";

// 默认导入
import Calculator from "./calculator.ll";

// 命名空间导入
import * as Math from "./math.ll";

// 重命名
import { add as addNumbers } from "./math.ll";
```

### 动态导入

```ziv
let module = await import("./module.ll");
module.doSomething();
```

---

## 标准库

### console

```ziv
console.log("Hello");
console.error("Error");
console.warn("Warning");
console.table([1, 2, 3]);
console.time("timer");
console.timeEnd("timer");
```

### Array

```ziv
let arr = [1, 2, 3, 4, 5];

arr.push(6);             // 添加元素
arr.pop();               // 移除最后一个
arr.shift();             // 移除第一个
arr.unshift(0);          // 添加到开头
arr.map(x => x * 2);     // 映射
arr.filter(x => x > 2);  // 过滤
arr.reduce((a, b) => a + b, 0);  // 归约
arr.sort((a, b) => a - b);       // 排序
arr.reverse();                   // 反转
arr.slice(1, 3);                 // 切片
arr.splice(2, 1);                // 删除
arr.indexOf(3);                  // 查找索引
arr.includes(3);                 // 包含
arr.find(x => x > 2);            // 查找元素
arr.findIndex(x => x > 2);       // 查找索引
```

### String

```ziv
let s = "  Hello, World!  ";

s.trim();                  // 去除空格
s.toLowerCase();           // 转小写
s.toUpperCase();           // 转大写
s.split(", ");             // 分割
s.replace("World", "LL");  // 替换
s.substring(0, 5);         // 子串
s.indexOf("World");        // 查找
s.includes("Hello");       // 包含
s.startsWith("  He");      // 以...开始
s.endsWith("!  ");         // 以...结束
s.length;                  // 长度
```

### Object

```ziv
let obj = { a: 1, b: 2, c: 3 };

Object.keys(obj);          // ["a", "b", "c"]
Object.values(obj);        // [1, 2, 3]
Object.entries(obj);       // [["a", 1], ["b", 2], ["c", 3]]
Object.assign({}, obj);    // 浅拷贝
Object.freeze(obj);        // 冻结
```

### Promise

```ziv
let promise = new Promise((resolve, reject) => {
    setTimeout(() => {
        resolve("done");
    }, 1000);
});

promise
    .then(result => console.log(result))
    .catch(error => console.error(error))
    .finally(() => console.log("cleanup"));

// async/await
async function fetch() {
    let response = await fetch(url);
    let data = await response.json();
    return data;
}
```

---

## 完整示例

### Hello World

```ziv
import console from "std";

function main(): int {
    console.log("Hello, World!");
    return 0;
}
```

### 斐波那契

```ziv
function fib(n: int): int {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

console.log(fib(10));  // 55
```

### 系统调用示例

```ziv
import syscall from "std/os";

fn main(): int {
    let msg = "Hello from LightLang!\n";
    syscall.write(1, msg.as_bytes());
    return 0;
}
```

### 类示例

```ziv
class Counter {
    private count: int = 0;
    
    increment(): void {
        this.count++;
    }
    
    get(): int {
        return this.count;
    }
}

let counter = new Counter();
counter.increment();
counter.increment();
console.log(counter.get());  // 2
```

---

## 编译器选项

```bash
# 编译为 ELF
llc hello.ll -o hello

# 优化级别
llc hello.ll -O0  # 无优化
llc hello.ll -O1  # 基本优化
llc hello.ll -O2  # 标准优化
llc hello.ll -O3  # 激进优化
llc hello.ll -Os  # 优化大小

# 目标架构
llc hello.ll --target x86_64
llc hello.ll --target aarch64
llc hello.ll --target riscv64

# 输出中间代码
llc hello.ll --emit-llvm -o hello.ll
llc hello.ll -S -o hello.s  # 汇编代码
```

---

## 与 JavaScript 的差异

| 特性 | LightLang | JavaScript |
|------|-----------|------------|
| 类型系统 | 静态 + 类型推断 | 动态 |
| 内存管理 | 区域推断 | GC |
| 编译目标 | ELF | JIT/字节码 |
| 变量提升 | ❌ | ✅ |
| null/undefined | 区分 | 不区分 |
| 多线程 | ✅ | Web Worker |

---

*LightLang v0.1 - JavaScript 的优雅，C 的性能*
