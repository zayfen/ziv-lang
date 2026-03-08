# Ziv 标准库 API（中文）

**版本**: 0.1.0  
**函数总数**: 117

## 说明

- 本文档由当前 `ziv-stdlib` 注册表导出。
- 表中“签名”是标准库元信息签名（用于符号注册与语义检查）。
- 当前默认运行时可执行全部 117 个函数。
- 其中 `net` / `crypto` / `js` 的部分行为为轻量实现，用于接口稳定和回归验证；生产级能力可通过宿主 runtime 替换。

## 目录

1. [io](#io)
2. [math](#math)
3. [string](#string)
4. [array](#array)
5. [container](#container)
6. [js](#js)
7. [filesystem](#filesystem)
8. [net](#net)
9. [crypto](#crypto)
10. [encoding](#encoding)

## io

| 函数 | 签名 | 返回类型 | 描述 |
|---|---|---|---|
| `eprint` | `eprint(value: any)` | `void` | Print a value to stderr without newline |
| `eprintln` | `eprintln(value: any)` | `void` | Print a value to stderr with newline |
| `flush` | `flush(-)` | `void` | Flush stdout buffer |
| `input` | `input(prompt: string)` | `string` | Read one line from stdin with prompt |
| `print` | `print(value: any)` | `void` | Print a value to stdout without newline |
| `printf` | `printf(format: string, value: any)` | `void` | Formatted output to stdout |
| `println` | `println(value: any)` | `void` | Print a value to stdout with newline |
| `read` | `read(-)` | `string` | Read a line from stdin |
| `readAll` | `readAll(-)` | `string` | Read all remaining stdin content |

## math

| 函数 | 签名 | 返回类型 | 描述 |
|---|---|---|---|
| `abs` | `abs(x: number)` | `number` | Return the absolute value of a number |
| `ceil` | `ceil(x: number)` | `i64` | Return the smallest integer greater than or equal to x |
| `floor` | `floor(x: number)` | `i64` | Return the largest integer less than or equal to x |
| `max` | `max(a: number, b: number)` | `number` | Return the maximum of two numbers |
| `min` | `min(a: number, b: number)` | `number` | Return the minimum of two numbers |
| `pow` | `pow(base: number, exp: number)` | `f64` | Return base raised to the power of exp |
| `round` | `round(x: number)` | `i64` | Round x to the nearest integer |
| `sqrt` | `sqrt(x: number)` | `f64` | Return the square root of a number |

## string

| 函数 | 签名 | 返回类型 | 描述 |
|---|---|---|---|
| `char_at` | `char_at(s: string, index: i64)` | `char` | Get character at specified index |
| `concat` | `concat(a: string, b: string)` | `string` | Concatenate two strings |
| `contains` | `contains(s: string, substr: string)` | `bool` | Check if string contains substring |
| `strlen` | `strlen(s: string)` | `i64` | Return the length of a string |
| `substr` | `substr(s: string, start: i64, length: i64)` | `string` | Return a substring from start with given length |
| `to_lower` | `to_lower(s: string)` | `string` | Convert string to lowercase |
| `to_upper` | `to_upper(s: string)` | `string` | Convert string to uppercase |
| `trim` | `trim(s: string)` | `string` | Remove leading and trailing whitespace |

## array

| 函数 | 签名 | 返回类型 | 描述 |
|---|---|---|---|
| `arrlen` | `arrlen(arr: array)` | `i64` | Return the length of an array |
| `first` | `first(arr: array)` | `any` | Get the first element of an array |
| `get` | `get(arr: array, index: i64)` | `any` | Get element at specified index |
| `last` | `last(arr: array)` | `any` | Get the last element of an array |
| `pop` | `pop(arr: array)` | `any` | Remove and return the last element of an array |
| `push` | `push(arr: array, element: any)` | `array` | Add an element to the end of an array |
| `reverse` | `reverse(arr: array)` | `array` | Reverse the order of elements in an array |
| `set` | `set(arr: array, index: i64, value: any)` | `array` | Set element at specified index |

## container

| 函数 | 签名 | 返回类型 | 描述 |
|---|---|---|---|
| `hashMapClear` | `hashMapClear(map: hashmap)` | `hashmap` | Remove all entries from hash map |
| `hashMapGet` | `hashMapGet(map: hashmap, key: any)` | `any` | Get value by key from hash map |
| `hashMapHas` | `hashMapHas(map: hashmap, key: any)` | `bool` | Return whether hash map has key |
| `hashMapKeys` | `hashMapKeys(map: hashmap)` | `array` | Return all hash map keys |
| `hashMapLen` | `hashMapLen(map: hashmap)` | `i64` | Return hash map size |
| `hashMapMerge` | `hashMapMerge(target: hashmap, source: hashmap)` | `hashmap` | Merge source hash map into target hash map |
| `hashMapNew` | `hashMapNew(-)` | `hashmap` | Create an empty hash map |
| `hashMapRemove` | `hashMapRemove(map: hashmap, key: any)` | `any` | Remove key from hash map and return old value |
| `hashMapSet` | `hashMapSet(map: hashmap, key: any, value: any)` | `hashmap` | Set key/value pair in hash map |
| `hashMapValues` | `hashMapValues(map: hashmap)` | `array` | Return all hash map values |
| `vectorClear` | `vectorClear(vec: vector)` | `vector` | Clear all vector elements |
| `vectorContains` | `vectorContains(vec: vector, value: any)` | `bool` | Return whether vector contains value |
| `vectorGet` | `vectorGet(vec: vector, index: i64)` | `any` | Get vector element at index |
| `vectorInsert` | `vectorInsert(vec: vector, index: i64, value: any)` | `vector` | Insert vector element at index |
| `vectorLen` | `vectorLen(vec: vector)` | `i64` | Return vector length |
| `vectorNew` | `vectorNew(-)` | `vector` | Create an empty vector |
| `vectorPop` | `vectorPop(vec: vector)` | `any` | Remove and return last vector element |
| `vectorPush` | `vectorPush(vec: vector, value: any)` | `vector` | Append value to vector |
| `vectorRemove` | `vectorRemove(vec: vector, index: i64)` | `any` | Remove vector element at index |
| `vectorSet` | `vectorSet(vec: vector, index: i64, value: any)` | `vector` | Set vector element at index |

## js

| 函数 | 签名 | 返回类型 | 描述 |
|---|---|---|---|
| `Boolean` | `Boolean(value: any)` | `bool` | Coerce value to boolean |
| `Number` | `Number(value: any)` | `number` | Coerce value to number |
| `String` | `String(value: any)` | `string` | Coerce value to string |
| `endsWith` | `endsWith(text: string, suffix: string)` | `bool` | Return whether text ends with suffix |
| `filter` | `filter(arr: array, fn: function)` | `array` | Filter array with callback |
| `includes` | `includes(text: string, search: string)` | `bool` | Check whether text includes search |
| `indexOf` | `indexOf(text: string, search: string)` | `i64` | Return index of search in text |
| `isFinite` | `isFinite(value: any)` | `bool` | Return whether value is finite |
| `isNaN` | `isNaN(value: any)` | `bool` | Return whether value is NaN-like |
| `jsonParse` | `jsonParse(text: string)` | `any` | Parse JSON text |
| `jsonStringify` | `jsonStringify(value: any)` | `string` | Serialize value to JSON |
| `map` | `map(arr: array, fn: function)` | `array` | Map array with callback |
| `parseFloat` | `parseFloat(text: string)` | `f64` | Parse a floating-point number from text |
| `parseInt` | `parseInt(text: string, radix: i64)` | `i64` | Parse an integer from text with optional radix |
| `reduce` | `reduce(arr: array, fn: function, initial: any)` | `any` | Reduce array with callback and initial value |
| `replace` | `replace(text: string, pattern: string, replacement: string)` | `string` | Replace first pattern in text |
| `split` | `split(text: string, sep: string)` | `array` | Split text by separator |
| `startsWith` | `startsWith(text: string, prefix: string)` | `bool` | Return whether text starts with prefix |

## filesystem

| 函数 | 签名 | 返回类型 | 描述 |
|---|---|---|---|
| `appendFile` | `appendFile(path: string, content: string)` | `bool` | Append text to file |
| `copyFile` | `copyFile(src: string, dst: string)` | `bool` | Copy file |
| `cwd` | `cwd(-)` | `string` | Return current working directory |
| `exists` | `exists(path: string)` | `bool` | Check whether path exists |
| `fileSize` | `fileSize(path: string)` | `i64` | Return file size in bytes |
| `mkdir` | `mkdir(path: string)` | `bool` | Create directory |
| `readDir` | `readDir(path: string)` | `array` | List directory entries |
| `readFile` | `readFile(path: string, encoding: string)` | `string` | Read text file content |
| `removeDir` | `removeDir(path: string)` | `bool` | Remove directory |
| `removeFile` | `removeFile(path: string)` | `bool` | Remove file |
| `rename` | `rename(from: string, to: string)` | `bool` | Rename file or directory |
| `writeFile` | `writeFile(path: string, content: string)` | `bool` | Write text to file |

## net

| 函数 | 签名 | 返回类型 | 描述 |
|---|---|---|---|
| `dnsLookup` | `dnsLookup(host: string)` | `string` | Resolve host to IP |
| `download` | `download(url: string, path: string)` | `bool` | Download URL content to file |
| `fetch` | `fetch(url: string)` | `string` | Perform HTTP GET and return body |
| `httpDelete` | `httpDelete(url: string)` | `string` | HTTP DELETE request |
| `httpGet` | `httpGet(url: string)` | `string` | HTTP GET request |
| `httpPost` | `httpPost(url: string, body: string)` | `string` | HTTP POST request |
| `httpPut` | `httpPut(url: string, body: string)` | `string` | HTTP PUT request |
| `ping` | `ping(host: string)` | `bool` | Check host reachability |
| `upload` | `upload(url: string, path: string)` | `string` | Upload file to URL |
| `websocketConnect` | `websocketConnect(url: string)` | `bool` | Connect websocket endpoint |

## crypto

| 函数 | 签名 | 返回类型 | 描述 |
|---|---|---|---|
| `decryptAES` | `decryptAES(ciphertext: string, key: string)` | `string` | Decrypt AES cipher text |
| `encryptAES` | `encryptAES(plaintext: string, key: string)` | `string` | Encrypt text with AES |
| `hmacSha256` | `hmacSha256(text: string, key: string)` | `string` | Compute HMAC-SHA256 |
| `md5` | `md5(text: string)` | `string` | Compute MD5 hash |
| `pbkdf2` | `pbkdf2(password: string, salt: string, iterations: i64)` | `string` | Derive key using PBKDF2 |
| `randomBytes` | `randomBytes(length: i64)` | `string` | Generate random bytes (hex/base64 text) |
| `randomUUID` | `randomUUID(-)` | `string` | Generate random UUID |
| `sha1` | `sha1(text: string)` | `string` | Compute SHA-1 hash |
| `sha256` | `sha256(text: string)` | `string` | Compute SHA-256 hash |
| `sha512` | `sha512(text: string)` | `string` | Compute SHA-512 hash |
| `sign` | `sign(message: string, privateKey: string)` | `string` | Sign message |
| `verify` | `verify(message: string, signature: string, publicKey: string)` | `bool` | Verify signature |

## encoding

| 函数 | 签名 | 返回类型 | 描述 |
|---|---|---|---|
| `base64Decode` | `base64Decode(base64: string)` | `string` | Decode Base64 text |
| `base64Encode` | `base64Encode(text: string)` | `string` | Encode text with Base64 |
| `csvDecode` | `csvDecode(text: string)` | `array` | Decode CSV text to rows |
| `csvEncode` | `csvEncode(rows: array)` | `string` | Encode rows to CSV text |
| `hexDecode` | `hexDecode(hex: string)` | `string` | Decode hex text |
| `hexEncode` | `hexEncode(text: string)` | `string` | Encode text as hex |
| `queryParse` | `queryParse(query: string)` | `any` | Parse URL query string |
| `queryStringify` | `queryStringify(obj: any)` | `string` | Serialize object as URL query string |
| `urlDecode` | `urlDecode(text: string)` | `string` | Decode percent-encoded text |
| `urlEncode` | `urlEncode(text: string)` | `string` | Percent-encode URL component |
| `utf8Decode` | `utf8Decode(bytes: array)` | `string` | Decode UTF-8 byte array to string |
| `utf8Encode` | `utf8Encode(text: string)` | `array` | Encode string to UTF-8 byte array |
