# Ziv 标准库介绍与使用指南

本文档讲解标准库的设计、使用方式、链接路径与当前运行时能力。

## 1. 标准库在哪里

标准库定义位于独立 crate：

- `crates/ziv-stdlib/`

编译器通过 `Stdlib::new()` 读取内置函数元信息（函数名、参数、返回类型、分类、描述）。

## 2. 标准库如何进入编译流程

### 2.1 语义阶段（符号注册）

语义分析器启动时会把标准库函数注册到符号表，后续源码中可直接调用。

效果：

- `print/println/abs/...` 在语义检查阶段被识别为已定义函数
- 参数和返回类型用于基础类型检查

### 2.2 IR 阶段（调用降级）

- `io`：`print/println/eprint/eprintln/read/input/readAll/printf/flush`
- `math`：`abs/min/max/sqrt/pow/floor/ceil/round`
- `string`：`strlen/concat/substr/char_at/to_upper/to_lower/trim/contains`
- `array`：`push/pop/arrlen/get/set/first/last/reverse`
- `container`：`vector*` / `hashMap*`
- `utils`：`parseInt/parseFloat/.../map/filter/reduce`
- `filesystem`：`readFile/writeFile/.../cwd`
- `net`：`fetch/http*/download/upload/dnsLookup/ping/websocketConnect`
- `crypto`：`md5/sha*/hmac/pbkdf2/encrypt/decrypt/sign/verify/random*`
- `encoding`：`base64/hex/url/utf8/csv/query`

以上函数在 IR 中都会降级为 runtime 符号调用，最终链接到可执行文件。

### 2.3 链接阶段（可执行文件）

编译器会生成并链接一个运行时对象（由内嵌 C runtime 编译得到），包含标准库函数实现，保证可执行文件可直接运行标准库调用。

## 3. 标准库分类

当前共 117 个函数，分为 10 类：

- `io`（9）
- `math`（8）
- `string`（8）
- `array`（8）
- `container`（20）
- `utils`（18）
- `filesystem`（12）
- `net`（10）
- `crypto`（12）
- `encoding`（12）

完整签名见：[STDLIB_API.md](STDLIB_API.md)

## 4. 使用方式

### 4.1 直接调用

```ziv
println("io demo");
abs(-10);
strlen("abc");
```

### 4.2 与结构体、函数混用

```ziv
struct User {
    age: int;
    score: int;
}

function show(u: User): int {
    println(u.age);
    return u.score;
}

let u: User = User.(age = 18, score = 90);
println(show(u));
```

### 4.3 函数参数传函数

```ziv
function inc(x: int): int { return x + 1; }
function apply(f: function, v: int): int { return f(v); }
println(apply(inc, 41));
```

## 5. 示例与验证

标准库示例目录：

- `examples/stdlib/hello.ziv`
- `examples/stdlib/io_demo.ziv`
- `examples/stdlib/math_demo.ziv`
- `examples/stdlib/string_demo.ziv`
- `examples/stdlib/array_demo.ziv`
- `examples/stdlib/container_demo.ziv`
- `examples/stdlib/utils_demo.ziv`
- `examples/stdlib/filesystem_demo.ziv`
- `examples/stdlib/net_demo.ziv`
- `examples/stdlib/crypto_demo.ziv`
- `examples/stdlib/encoding_demo.ziv`

运行全部测试：

```bash
cargo test --workspace --all-targets
```

单独验证示例：

```bash
./target/debug/ziv examples/stdlib/hello.ziv -o /tmp/hello && /tmp/hello
```

批量验证 stdlib 示例：

```bash
for f in examples/stdlib/*.ziv; do
  ./target/debug/ziv "$f" -o /tmp/ziv_example && /tmp/ziv_example </dev/null
done
```

## 6. 当前实现边界

- 117 个标准库函数均可执行、可编译、可测试。
- `net` / `crypto` / `utils` 的部分函数当前为轻量实现（接口稳定、结果可预测），优先服务语言回归测试与示例验证。
- 若业务需要完整生产能力（例如真实 HTTP 客户端栈、严格密码学实现、复杂 JSON 语义），建议替换为外部链接 runtime 或宿主库实现。

## 7. 推荐实践

- 把标准库函数当作稳定 API 面设计代码
- 对外部副作用函数（文件/网络/加密）同时保留“语义回归测试”和“宿主集成测试”两层验证
- 在 examples 中保留可观测输出与副作用校验点，保证 CI 能验证真实行为而不是仅检查可编译
