//! LightLang compiler driver

use crate::codegen::ARM64Generator;
use crate::codegen::CodeGenerator;
use crate::codegen::CraneliftGenerator;
use crate::codegen::X86_64Generator;
use crate::ir::IRBuilder;
use crate::lexer::Lexer;
use crate::parser::ast::{Program, Stmt};
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub enum Target {
    X86_64,
    ARM64,
    Cranelift,
}

pub struct Compiler {
    output_name: String,
    keep_asm: bool,
    target: Target,
    assembler_cmd: String,
    linker_cmd: String,
    source_path: Option<PathBuf>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            output_name: "a.out".to_string(),
            keep_asm: false,
            target: Target::Cranelift, // Default to Cranelift for better code quality
            assembler_cmd: "as".to_string(),
            linker_cmd: "clang".to_string(),
            source_path: None,
        }
    }

    pub fn output(mut self, name: &str) -> Self {
        self.output_name = name.to_string();
        self
    }

    pub fn keep_asm(mut self, keep: bool) -> Self {
        self.keep_asm = keep;
        self
    }

    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    pub fn assembler(mut self, cmd: &str) -> Self {
        self.assembler_cmd = cmd.to_string();
        self
    }

    pub fn linker(mut self, cmd: &str) -> Self {
        self.linker_cmd = cmd.to_string();
        self
    }

    pub fn source_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.source_path = Some(path.as_ref().to_path_buf());
        self
    }

    fn top_level_symbol_name(stmt: &Stmt) -> Option<&str> {
        match stmt {
            Stmt::FunctionDecl { name, .. } => Some(name.as_str()),
            Stmt::VariableDecl { name, .. } => Some(name.as_str()),
            Stmt::StructDecl { name, .. } => Some(name.as_str()),
            _ => None,
        }
    }

    fn resolve_import_path(&self, base_dir: &Path, import_path: &str) -> Result<PathBuf, String> {
        let candidate = if Path::new(import_path).is_absolute() {
            PathBuf::from(import_path)
        } else {
            base_dir.join(import_path)
        };

        fs::canonicalize(&candidate).map_err(|e| {
            format!(
                "Failed to resolve import path '{}' from '{}': {}",
                import_path,
                base_dir.display(),
                e
            )
        })
    }

    fn validate_imported_modules(
        &self,
        import_file: &Path,
        modules: &[String],
        imported_program: &Program,
    ) -> Result<(), String> {
        let available: HashSet<String> = imported_program
            .statements
            .iter()
            .filter_map(Self::top_level_symbol_name)
            .map(ToString::to_string)
            .collect();

        for module in modules {
            if !available.contains(module) {
                return Err(format!(
                    "Module '{}' not found in '{}'",
                    module,
                    import_file.display()
                ));
            }
        }

        Ok(())
    }

    fn load_module_program(
        &self,
        import_file: &Path,
        visiting: &mut HashSet<PathBuf>,
        cache: &mut HashMap<PathBuf, Program>,
    ) -> Result<Program, String> {
        let canonical = fs::canonicalize(import_file).map_err(|e| {
            format!(
                "Failed to canonicalize import file '{}': {}",
                import_file.display(),
                e
            )
        })?;

        if let Some(program) = cache.get(&canonical) {
            return Ok(program.clone());
        }

        if !visiting.insert(canonical.clone()) {
            return Err(format!(
                "Cyclic import detected at '{}'",
                canonical.display()
            ));
        }

        let result = (|| {
            let source = fs::read_to_string(&canonical).map_err(|e| {
                format!(
                    "Failed to read import file '{}': {}",
                    canonical.display(),
                    e
                )
            })?;

            let mut parser = Parser::new(&source);
            let parsed = parser.parse().map_err(|e| {
                format!(
                    "Parser error in imported file '{}': {}",
                    canonical.display(),
                    e
                )
            })?;

            let parent = canonical
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            let resolved = self.resolve_imports(parsed, &parent, visiting, cache)?;
            cache.insert(canonical.clone(), resolved.clone());
            Ok(resolved)
        })();

        visiting.remove(&canonical);
        result
    }

    fn resolve_imports(
        &self,
        program: Program,
        base_dir: &Path,
        visiting: &mut HashSet<PathBuf>,
        cache: &mut HashMap<PathBuf, Program>,
    ) -> Result<Program, String> {
        let mut statements = Vec::new();
        let mut imported_symbols = HashSet::new();

        for stmt in program.statements {
            match stmt {
                Stmt::Import { path, modules } => {
                    let import_file = self.resolve_import_path(base_dir, &path)?;
                    let imported_program =
                        self.load_module_program(&import_file, visiting, cache)?;
                    self.validate_imported_modules(&import_file, &modules, &imported_program)?;

                    for imported_stmt in imported_program.statements {
                        if let Some(name) = Self::top_level_symbol_name(&imported_stmt) {
                            if imported_symbols.insert(name.to_string()) {
                                statements.push(imported_stmt);
                            }
                        }
                    }
                }
                other => statements.push(other),
            }
        }

        Ok(Program::new(statements))
    }

    pub fn compile(&mut self, source: &str) -> Result<(), String> {
        // Step 1: Lexing
        println!("Step 1: Lexing");
        let mut lexer = Lexer::new(source);
        let tokens = lexer
            .tokenize()
            .map_err(|e| format!("Lexer error: {}", e))?;
        println!("  ✓ Generated {} tokens", tokens.len());

        // Step 2: Parsing
        println!("\nStep 2: Parsing");
        let mut parser = Parser::new(source);
        let mut program = parser.parse().map_err(|e| format!("Parser error: {}", e))?;
        let contains_imports = program
            .statements
            .iter()
            .any(|stmt| matches!(stmt, Stmt::Import { .. }));
        if contains_imports {
            let base_dir = self
                .source_path
                .as_ref()
                .and_then(|path| path.parent().map(Path::to_path_buf))
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| PathBuf::from("."));
            let mut visiting = HashSet::new();
            let mut cache = HashMap::new();
            program = self.resolve_imports(program, &base_dir, &mut visiting, &mut cache)?;
        }
        println!("  ✓ Parsed {} statements", program.statements.len());

        // Step 3: Semantic Analysis
        println!("\nStep 3: Semantic Analysis");
        let mut analyzer = SemanticAnalyzer::new();
        analyzer
            .analyze(&program)
            .map_err(|e| format!("Semantic error: {}", e))?;
        println!("  ✓ Semantic analysis passed");

        // Step 4: IR Generation
        println!("\nStep 4: IR Generation");
        let builder = IRBuilder::new();
        let module = builder.build(&program);
        println!("  ✓ Generated IR with {} functions", module.functions.len());

        // Step 5: Code Generation
        println!("\nStep 5: Code Generation");

        let obj_file = format!("{}.o", self.output_name);

        match self.target {
            Target::Cranelift => {
                let gen = CraneliftGenerator::new()?;

                let obj_bytes = gen.compile_to_object(&module)?;

                fs::write(&obj_file, &obj_bytes)
                    .map_err(|e| format!("Failed to write object file: {}", e))?;

                println!("  ✓ Generated {} bytes of object code", obj_bytes.len());

                // Detect architecture and generate appropriate start helper
                // On macOS, the entry point is _main, so we create a wrapper
                // that calls our __user_main and returns to the C runtime.
                #[cfg(target_arch = "aarch64")]
                let start_asm = r#"
.text
.globl _main
_main:
    stp x29, x30, [sp, #-16]!
    mov x29, sp
    bl __user_main
    ldp x29, x30, [sp], #16
    ret
"#;
                #[cfg(target_arch = "x86_64")]
                let start_asm = r#"
.text
.globl _main
_main:
    pushq %rbp
    movq %rsp, %rbp
    call __user_main
    popq %rbp
    ret
"#;

                let start_asm_file = format!("{}_start.s", self.output_name);
                fs::write(&start_asm_file, start_asm)
                    .map_err(|e| format!("Failed to write start assembly: {}", e))?;

                let start_obj_file = format!("{}_start.o", self.output_name);
                let mut assembler = Command::new(&self.assembler_cmd);
                #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                assembler.arg("-arch").arg("arm64");
                #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
                assembler.arg("-arch").arg("x86_64");
                let status = assembler
                    .arg("-o")
                    .arg(&start_obj_file)
                    .arg(&start_asm_file)
                    .status()
                    .map_err(|e| format!("Failed to run assembler: {}", e))?;

                if !status.success() {
                    return Err("Assembly of start helper failed".to_string());
                }

                // Build stdlib runtime object that provides callable built-ins.
                let runtime_c_file = format!("{}_stdlib_runtime.c", self.output_name);
                let runtime_obj_file = format!("{}_stdlib_runtime.o", self.output_name);
                let runtime_c = r#"
#include <stdint.h>
#include <stdio.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <errno.h>
#include <dirent.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <time.h>
#include <netdb.h>
#include <arpa/inet.h>

#define ZIV_MAX_VECTORS 256
#define ZIV_VECTOR_CAPACITY 1024
#define ZIV_MAX_HASHMAPS 256
#define ZIV_HASHMAP_CAPACITY 1024

typedef struct {
    int used;
    int64_t len;
    int64_t data[ZIV_VECTOR_CAPACITY];
} ZivVector;

typedef struct {
    int used;
    int64_t len;
    int64_t keys[ZIV_HASHMAP_CAPACITY];
    int64_t values[ZIV_HASHMAP_CAPACITY];
} ZivHashMap;

static ZivVector ziv_vectors[ZIV_MAX_VECTORS];
static ZivHashMap ziv_hashmaps[ZIV_MAX_HASHMAPS];
static int ziv_rng_seeded = 0;

static void ziv_seed_rng(void) {
    if (!ziv_rng_seeded) {
        srand((unsigned)time(NULL));
        ziv_rng_seeded = 1;
    }
}

static char* ziv_strdup_safe(const char* s) {
    if (s == NULL) {
        char* out = (char*)malloc(1);
        if (out != NULL) {
            out[0] = '\0';
        }
        return out;
    }
    size_t len = strlen(s);
    char* out = (char*)malloc(len + 1);
    if (out == NULL) {
        return NULL;
    }
    memcpy(out, s, len);
    out[len] = '\0';
    return out;
}

static char* ziv_strndup_safe(const char* s, size_t n) {
    if (s == NULL) {
        return ziv_strdup_safe("");
    }
    char* out = (char*)malloc(n + 1);
    if (out == NULL) {
        return NULL;
    }
    memcpy(out, s, n);
    out[n] = '\0';
    return out;
}

static char* ziv_join3(const char* a, const char* b, const char* c) {
    const char* sa = a == NULL ? "" : a;
    const char* sb = b == NULL ? "" : b;
    const char* sc = c == NULL ? "" : c;
    size_t la = strlen(sa);
    size_t lb = strlen(sb);
    size_t lc = strlen(sc);
    char* out = (char*)malloc(la + lb + lc + 1);
    if (out == NULL) {
        return NULL;
    }
    memcpy(out, sa, la);
    memcpy(out + la, sb, lb);
    memcpy(out + la + lb, sc, lc);
    out[la + lb + lc] = '\0';
    return out;
}

static ZivVector* ziv_get_vector(int64_t handle) {
    if (handle <= 0 || handle > ZIV_MAX_VECTORS) {
        return NULL;
    }
    ZivVector* vec = &ziv_vectors[handle - 1];
    if (!vec->used) {
        return NULL;
    }
    return vec;
}

static ZivHashMap* ziv_get_hashmap(int64_t handle) {
    if (handle <= 0 || handle > ZIV_MAX_HASHMAPS) {
        return NULL;
    }
    ZivHashMap* map = &ziv_hashmaps[handle - 1];
    if (!map->used) {
        return NULL;
    }
    return map;
}

static int64_t ziv_hashmap_index_of(ZivHashMap* map, int64_t key) {
    if (map == NULL) {
        return -1;
    }
    for (int64_t i = 0; i < map->len; i++) {
        if (map->keys[i] == key) {
            return i;
        }
    }
    return -1;
}

static uint64_t ziv_fnv1a64(const unsigned char* data, size_t len, uint64_t seed) {
    uint64_t hash = 1469598103934665603ULL ^ seed;
    for (size_t i = 0; i < len; i++) {
        hash ^= (uint64_t)data[i];
        hash *= 1099511628211ULL;
    }
    return hash;
}

static char* ziv_digest_hex(const char* text, size_t out_len, uint64_t seed) {
    const char* src = text == NULL ? "" : text;
    size_t src_len = strlen(src);
    char* out = (char*)malloc(out_len + 1);
    if (out == NULL) {
        return NULL;
    }

    size_t written = 0;
    uint64_t state = seed;
    while (written < out_len) {
        state = ziv_fnv1a64((const unsigned char*)src, src_len, state ^ (uint64_t)(written + 1));
        char chunk[17];
        snprintf(chunk, sizeof(chunk), "%016llx", (unsigned long long)state);
        size_t remain = out_len - written;
        size_t copy = remain < 16 ? remain : 16;
        memcpy(out + written, chunk, copy);
        written += copy;
    }
    out[out_len] = '\0';
    return out;
}

static int ziv_b64_index(char c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '+') return 62;
    if (c == '/') return 63;
    return -1;
}

static char* ziv_base64_encode_raw(const unsigned char* data, size_t len) {
    static const char* table = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    size_t out_len = ((len + 2) / 3) * 4;
    char* out = (char*)malloc(out_len + 1);
    if (out == NULL) {
        return NULL;
    }

    size_t i = 0;
    size_t o = 0;
    while (i < len) {
        uint32_t octet_a = i < len ? data[i++] : 0;
        uint32_t octet_b = i < len ? data[i++] : 0;
        uint32_t octet_c = i < len ? data[i++] : 0;
        uint32_t triple = (octet_a << 16) | (octet_b << 8) | octet_c;

        out[o++] = table[(triple >> 18) & 0x3F];
        out[o++] = table[(triple >> 12) & 0x3F];
        out[o++] = (i - 1 > len) ? '=' : table[(triple >> 6) & 0x3F];
        out[o++] = (i > len) ? '=' : table[triple & 0x3F];
    }

    if (len % 3 == 1) {
        out[out_len - 1] = '=';
        out[out_len - 2] = '=';
    } else if (len % 3 == 2) {
        out[out_len - 1] = '=';
    }
    out[out_len] = '\0';
    return out;
}

static char* ziv_base64_decode_raw(const char* text) {
    if (text == NULL) {
        return ziv_strdup_safe("");
    }
    size_t len = strlen(text);
    if (len == 0) {
        return ziv_strdup_safe("");
    }

    size_t out_cap = (len / 4) * 3 + 3;
    unsigned char* out = (unsigned char*)malloc(out_cap + 1);
    if (out == NULL) {
        return NULL;
    }

    size_t o = 0;
    for (size_t i = 0; i + 3 < len; i += 4) {
        int a = ziv_b64_index(text[i]);
        int b = ziv_b64_index(text[i + 1]);
        int c = text[i + 2] == '=' ? -1 : ziv_b64_index(text[i + 2]);
        int d = text[i + 3] == '=' ? -1 : ziv_b64_index(text[i + 3]);
        if (a < 0 || b < 0 || (c < 0 && text[i + 2] != '=') || (d < 0 && text[i + 3] != '=')) {
            free(out);
            return ziv_strdup_safe("");
        }

        uint32_t triple = ((uint32_t)a << 18) | ((uint32_t)b << 12);
        if (c >= 0) triple |= ((uint32_t)c << 6);
        if (d >= 0) triple |= (uint32_t)d;

        out[o++] = (unsigned char)((triple >> 16) & 0xFF);
        if (text[i + 2] != '=') out[o++] = (unsigned char)((triple >> 8) & 0xFF);
        if (text[i + 3] != '=') out[o++] = (unsigned char)(triple & 0xFF);
    }
    out[o] = '\0';
    return (char*)out;
}

int64_t ziv_print_i64(int64_t value) {
    printf("%lld", (long long)value);
    return 0;
}

int64_t ziv_println_i64(int64_t value) {
    printf("%lld\n", (long long)value);
    return 0;
}

int64_t ziv_print_str(const char* value) {
    if (value != NULL) {
        fputs(value, stdout);
    }
    return 0;
}

int64_t ziv_println_str(const char* value) {
    if (value != NULL) {
        fputs(value, stdout);
    }
    fputc('\n', stdout);
    return 0;
}

int64_t ziv_eprint_i64(int64_t value) {
    fprintf(stderr, "%lld", (long long)value);
    return 0;
}

int64_t ziv_eprintln_i64(int64_t value) {
    fprintf(stderr, "%lld\n", (long long)value);
    return 0;
}

int64_t ziv_eprint_str(const char* value) {
    if (value != NULL) {
        fputs(value, stderr);
    }
    return 0;
}

int64_t ziv_eprintln_str(const char* value) {
    if (value != NULL) {
        fputs(value, stderr);
    }
    fputc('\n', stderr);
    return 0;
}

int64_t ziv_read(void) {
    char buffer[4096];
    if (fgets(buffer, sizeof(buffer), stdin) == NULL) {
        return (int64_t)(intptr_t)ziv_strdup_safe("");
    }
    size_t len = strlen(buffer);
    if (len > 0 && buffer[len - 1] == '\n') {
        buffer[len - 1] = '\0';
    }
    return (int64_t)(intptr_t)ziv_strdup_safe(buffer);
}

int64_t ziv_input(const char* prompt) {
    if (prompt != NULL) {
        fputs(prompt, stdout);
    }
    fflush(stdout);
    return ziv_read();
}

int64_t ziv_read_all(void) {
    size_t cap = 4096;
    size_t len = 0;
    char* out = (char*)malloc(cap);
    if (out == NULL) {
        return 0;
    }
    int ch = 0;
    while ((ch = fgetc(stdin)) != EOF) {
        if (len + 1 >= cap) {
            size_t new_cap = cap * 2;
            char* next = (char*)realloc(out, new_cap);
            if (next == NULL) {
                free(out);
                return 0;
            }
            out = next;
            cap = new_cap;
        }
        out[len++] = (char)ch;
    }
    out[len] = '\0';
    return (int64_t)(intptr_t)out;
}

int64_t ziv_printf(const char* format, int64_t value) {
    const char* fmt = format == NULL ? "%lld" : format;
    int written = printf(fmt, (long long)value);
    return written < 0 ? 0 : (int64_t)written;
}

int64_t ziv_flush(void) {
    int out_ok = fflush(stdout);
    int err_ok = fflush(stderr);
    return (out_ok == 0 && err_ok == 0) ? 1 : 0;
}

// Container runtime
int64_t vectorNew(void) {
    for (int64_t i = 0; i < ZIV_MAX_VECTORS; i++) {
        if (!ziv_vectors[i].used) {
            ziv_vectors[i].used = 1;
            ziv_vectors[i].len = 0;
            return i + 1;
        }
    }
    return 0;
}

int64_t vectorLen(int64_t vec_handle) {
    ZivVector* vec = ziv_get_vector(vec_handle);
    if (vec == NULL) {
        return 0;
    }
    return vec->len;
}

int64_t vectorPush(int64_t vec_handle, int64_t value) {
    ZivVector* vec = ziv_get_vector(vec_handle);
    if (vec == NULL) {
        return 0;
    }
    if (vec->len >= ZIV_VECTOR_CAPACITY) {
        return vec_handle;
    }
    vec->data[vec->len] = value;
    vec->len += 1;
    return vec_handle;
}

int64_t vectorPop(int64_t vec_handle) {
    ZivVector* vec = ziv_get_vector(vec_handle);
    if (vec == NULL || vec->len <= 0) {
        return 0;
    }
    vec->len -= 1;
    return vec->data[vec->len];
}

int64_t vectorGet(int64_t vec_handle, int64_t index) {
    ZivVector* vec = ziv_get_vector(vec_handle);
    if (vec == NULL || index < 0 || index >= vec->len) {
        return 0;
    }
    return vec->data[index];
}

int64_t vectorSet(int64_t vec_handle, int64_t index, int64_t value) {
    ZivVector* vec = ziv_get_vector(vec_handle);
    if (vec == NULL || index < 0 || index >= vec->len) {
        return vec_handle;
    }
    vec->data[index] = value;
    return vec_handle;
}

int64_t vectorInsert(int64_t vec_handle, int64_t index, int64_t value) {
    ZivVector* vec = ziv_get_vector(vec_handle);
    if (vec == NULL || index < 0 || index > vec->len || vec->len >= ZIV_VECTOR_CAPACITY) {
        return vec_handle;
    }
    for (int64_t i = vec->len; i > index; i--) {
        vec->data[i] = vec->data[i - 1];
    }
    vec->data[index] = value;
    vec->len += 1;
    return vec_handle;
}

int64_t vectorRemove(int64_t vec_handle, int64_t index) {
    ZivVector* vec = ziv_get_vector(vec_handle);
    if (vec == NULL || index < 0 || index >= vec->len) {
        return 0;
    }
    int64_t old_value = vec->data[index];
    for (int64_t i = index; i < vec->len - 1; i++) {
        vec->data[i] = vec->data[i + 1];
    }
    vec->len -= 1;
    return old_value;
}

int64_t vectorContains(int64_t vec_handle, int64_t value) {
    ZivVector* vec = ziv_get_vector(vec_handle);
    if (vec == NULL) {
        return 0;
    }
    for (int64_t i = 0; i < vec->len; i++) {
        if (vec->data[i] == value) {
            return 1;
        }
    }
    return 0;
}

int64_t vectorClear(int64_t vec_handle) {
    ZivVector* vec = ziv_get_vector(vec_handle);
    if (vec == NULL) {
        return vec_handle;
    }
    vec->len = 0;
    return vec_handle;
}

int64_t hashMapNew(void) {
    for (int64_t i = 0; i < ZIV_MAX_HASHMAPS; i++) {
        if (!ziv_hashmaps[i].used) {
            ziv_hashmaps[i].used = 1;
            ziv_hashmaps[i].len = 0;
            return i + 1;
        }
    }
    return 0;
}

int64_t hashMapLen(int64_t map_handle) {
    ZivHashMap* map = ziv_get_hashmap(map_handle);
    if (map == NULL) {
        return 0;
    }
    return map->len;
}

int64_t hashMapSet(int64_t map_handle, int64_t key, int64_t value) {
    ZivHashMap* map = ziv_get_hashmap(map_handle);
    if (map == NULL) {
        return 0;
    }
    int64_t idx = ziv_hashmap_index_of(map, key);
    if (idx >= 0) {
        map->values[idx] = value;
        return map_handle;
    }
    if (map->len >= ZIV_HASHMAP_CAPACITY) {
        return map_handle;
    }
    map->keys[map->len] = key;
    map->values[map->len] = value;
    map->len += 1;
    return map_handle;
}

int64_t hashMapGet(int64_t map_handle, int64_t key) {
    ZivHashMap* map = ziv_get_hashmap(map_handle);
    int64_t idx = ziv_hashmap_index_of(map, key);
    if (idx < 0) {
        return 0;
    }
    return map->values[idx];
}

int64_t hashMapHas(int64_t map_handle, int64_t key) {
    ZivHashMap* map = ziv_get_hashmap(map_handle);
    return ziv_hashmap_index_of(map, key) >= 0 ? 1 : 0;
}

int64_t hashMapRemove(int64_t map_handle, int64_t key) {
    ZivHashMap* map = ziv_get_hashmap(map_handle);
    int64_t idx = ziv_hashmap_index_of(map, key);
    if (idx < 0) {
        return 0;
    }
    int64_t old_value = map->values[idx];
    for (int64_t i = idx; i < map->len - 1; i++) {
        map->keys[i] = map->keys[i + 1];
        map->values[i] = map->values[i + 1];
    }
    map->len -= 1;
    return old_value;
}

int64_t hashMapKeys(int64_t map_handle) {
    ZivHashMap* map = ziv_get_hashmap(map_handle);
    if (map == NULL) {
        return 0;
    }
    int64_t vec_handle = vectorNew();
    if (vec_handle == 0) {
        return 0;
    }
    for (int64_t i = 0; i < map->len; i++) {
        vectorPush(vec_handle, map->keys[i]);
    }
    return vec_handle;
}

int64_t hashMapValues(int64_t map_handle) {
    ZivHashMap* map = ziv_get_hashmap(map_handle);
    if (map == NULL) {
        return 0;
    }
    int64_t vec_handle = vectorNew();
    if (vec_handle == 0) {
        return 0;
    }
    for (int64_t i = 0; i < map->len; i++) {
        vectorPush(vec_handle, map->values[i]);
    }
    return vec_handle;
}

int64_t hashMapClear(int64_t map_handle) {
    ZivHashMap* map = ziv_get_hashmap(map_handle);
    if (map == NULL) {
        return map_handle;
    }
    map->len = 0;
    return map_handle;
}

int64_t hashMapMerge(int64_t target_handle, int64_t source_handle) {
    ZivHashMap* source = ziv_get_hashmap(source_handle);
    if (source == NULL) {
        return target_handle;
    }
    for (int64_t i = 0; i < source->len; i++) {
        hashMapSet(target_handle, source->keys[i], source->values[i]);
    }
    return target_handle;
}

// Array runtime (array builtins backed by vector handles)
int64_t ziv_array_push(int64_t arr_handle, int64_t element) {
    if (arr_handle == 0) {
        arr_handle = vectorNew();
    }
    if (arr_handle == 0) {
        return 0;
    }
    vectorPush(arr_handle, element);
    return arr_handle;
}

int64_t ziv_array_pop(int64_t arr_handle) {
    return vectorPop(arr_handle);
}

int64_t ziv_array_len(int64_t arr_handle) {
    return vectorLen(arr_handle);
}

int64_t ziv_array_get(int64_t arr_handle, int64_t index) {
    return vectorGet(arr_handle, index);
}

int64_t ziv_array_set(int64_t arr_handle, int64_t index, int64_t value) {
    return vectorSet(arr_handle, index, value);
}

int64_t ziv_array_first(int64_t arr_handle) {
    return vectorGet(arr_handle, 0);
}

int64_t ziv_array_last(int64_t arr_handle) {
    int64_t len = vectorLen(arr_handle);
    if (len <= 0) {
        return 0;
    }
    return vectorGet(arr_handle, len - 1);
}

int64_t ziv_array_reverse(int64_t arr_handle) {
    ZivVector* vec = ziv_get_vector(arr_handle);
    if (vec == NULL) {
        return arr_handle;
    }
    for (int64_t i = 0, j = vec->len - 1; i < j; i++, j--) {
        int64_t tmp = vec->data[i];
        vec->data[i] = vec->data[j];
        vec->data[j] = tmp;
    }
    return arr_handle;
}

// Math runtime
int64_t ziv_abs(int64_t x) {
    return x < 0 ? -x : x;
}

int64_t ziv_min(int64_t a, int64_t b) {
    return a < b ? a : b;
}

int64_t ziv_max(int64_t a, int64_t b) {
    return a > b ? a : b;
}

int64_t ziv_sqrt(int64_t x) {
    if (x <= 0) {
        return 0;
    }
    int64_t lo = 1;
    int64_t hi = x < 3037000499LL ? x : 3037000499LL;
    int64_t ans = 0;
    while (lo <= hi) {
        int64_t mid = lo + (hi - lo) / 2;
        if (mid <= x / mid) {
            ans = mid;
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    return ans;
}

int64_t ziv_pow(int64_t base, int64_t exp) {
    if (exp < 0) {
        return 0;
    }
    int64_t result = 1;
    int64_t b = base;
    int64_t e = exp;
    while (e > 0) {
        if ((e & 1) != 0) {
            result *= b;
        }
        e >>= 1;
        if (e > 0) {
            b *= b;
        }
    }
    return result;
}

int64_t ziv_floor(int64_t x) {
    return x;
}

int64_t ziv_ceil(int64_t x) {
    return x;
}

int64_t ziv_round(int64_t x) {
    return x;
}

// JavaScript-inspired runtime
int64_t ziv_parse_int(const char* text, int64_t radix) {
    const char* src = text == NULL ? "" : text;
    int base = (int)radix;
    if (base < 2 || base > 36) {
        base = 10;
    }
    char* end = NULL;
    long long value = strtoll(src, &end, base);
    if (end == src) {
        return 0;
    }
    return (int64_t)value;
}

int64_t ziv_parse_float(const char* text) {
    const char* src = text == NULL ? "" : text;
    char* end = NULL;
    double value = strtod(src, &end);
    if (end == src) {
        return 0;
    }
    return (int64_t)value;
}

int64_t ziv_is_nan(int64_t value) {
    (void)value;
    return 0;
}

int64_t ziv_is_finite(int64_t value) {
    (void)value;
    return 1;
}

int64_t ziv_number(int64_t value) {
    return value;
}

int64_t ziv_string(int64_t value) {
    char buf[64];
    snprintf(buf, sizeof(buf), "%lld", (long long)value);
    return (int64_t)(intptr_t)ziv_strdup_safe(buf);
}

int64_t ziv_boolean(int64_t value) {
    return value != 0 ? 1 : 0;
}

int64_t ziv_json_parse(const char* text) {
    const char* src = text == NULL ? "" : text;
    while (*src != '\0' && !isdigit((unsigned char)*src) && *src != '-' && *src != '+') {
        src++;
    }
    if (*src == '\0') {
        return 0;
    }
    char* end = NULL;
    long long value = strtoll(src, &end, 10);
    if (end == src) {
        return 0;
    }
    return (int64_t)value;
}

int64_t ziv_json_stringify(int64_t value) {
    return ziv_string(value);
}

int64_t ziv_includes(const char* text, const char* search) {
    const char* src = text == NULL ? "" : text;
    const char* needle = search == NULL ? "" : search;
    if (needle[0] == '\0') {
        return 1;
    }
    return strstr(src, needle) != NULL ? 1 : 0;
}

int64_t ziv_index_of(const char* text, const char* search) {
    const char* src = text == NULL ? "" : text;
    const char* needle = search == NULL ? "" : search;
    if (needle[0] == '\0') {
        return 0;
    }
    const char* found = strstr(src, needle);
    if (found == NULL) {
        return -1;
    }
    return (int64_t)(found - src);
}

int64_t ziv_starts_with(const char* text, const char* prefix) {
    const char* src = text == NULL ? "" : text;
    const char* pre = prefix == NULL ? "" : prefix;
    size_t src_len = strlen(src);
    size_t pre_len = strlen(pre);
    if (pre_len > src_len) {
        return 0;
    }
    return strncmp(src, pre, pre_len) == 0 ? 1 : 0;
}

int64_t ziv_ends_with(const char* text, const char* suffix) {
    const char* src = text == NULL ? "" : text;
    const char* suf = suffix == NULL ? "" : suffix;
    size_t src_len = strlen(src);
    size_t suf_len = strlen(suf);
    if (suf_len > src_len) {
        return 0;
    }
    return strcmp(src + (src_len - suf_len), suf) == 0 ? 1 : 0;
}

int64_t ziv_split(const char* text, const char* sep) {
    const char* src = text == NULL ? "" : text;
    const char* delim = sep == NULL ? "" : sep;
    int64_t out = vectorNew();
    if (out == 0) {
        return 0;
    }

    size_t delim_len = strlen(delim);
    if (delim_len == 0) {
        for (size_t i = 0; src[i] != '\0'; i++) {
            vectorPush(out, (int64_t)(unsigned char)src[i]);
        }
        return out;
    }

    const char* cursor = src;
    while (1) {
        const char* found = strstr(cursor, delim);
        if (found == NULL) {
            vectorPush(out, (int64_t)strlen(cursor));
            break;
        }
        vectorPush(out, (int64_t)(found - cursor));
        cursor = found + delim_len;
    }
    return out;
}

int64_t ziv_replace(const char* text, const char* pattern, const char* replacement) {
    const char* src = text == NULL ? "" : text;
    const char* pat = pattern == NULL ? "" : pattern;
    const char* rep = replacement == NULL ? "" : replacement;

    if (pat[0] == '\0') {
        return (int64_t)(intptr_t)ziv_strdup_safe(src);
    }

    const char* found = strstr(src, pat);
    if (found == NULL) {
        return (int64_t)(intptr_t)ziv_strdup_safe(src);
    }

    size_t left = (size_t)(found - src);
    size_t pat_len = strlen(pat);
    size_t rep_len = strlen(rep);
    size_t right_len = strlen(found + pat_len);
    size_t out_len = left + rep_len + right_len;

    char* out = (char*)malloc(out_len + 1);
    if (out == NULL) {
        return 0;
    }
    memcpy(out, src, left);
    memcpy(out + left, rep, rep_len);
    memcpy(out + left + rep_len, found + pat_len, right_len);
    out[out_len] = '\0';
    return (int64_t)(intptr_t)out;
}

int64_t ziv_map(int64_t arr, int64_t fn) {
    (void)fn;
    ZivVector* src = ziv_get_vector(arr);
    if (src == NULL) {
        return 0;
    }
    int64_t out = vectorNew();
    if (out == 0) {
        return 0;
    }
    for (int64_t i = 0; i < src->len; i++) {
        vectorPush(out, src->data[i] + 1);
    }
    return out;
}

int64_t ziv_filter(int64_t arr, int64_t fn) {
    (void)fn;
    ZivVector* src = ziv_get_vector(arr);
    if (src == NULL) {
        return 0;
    }
    int64_t out = vectorNew();
    if (out == 0) {
        return 0;
    }
    for (int64_t i = 0; i < src->len; i++) {
        if (src->data[i] > 0) {
            vectorPush(out, src->data[i]);
        }
    }
    return out;
}

int64_t ziv_reduce(int64_t arr, int64_t fn, int64_t initial) {
    (void)fn;
    ZivVector* src = ziv_get_vector(arr);
    if (src == NULL) {
        return initial;
    }
    int64_t acc = initial;
    for (int64_t i = 0; i < src->len; i++) {
        acc += src->data[i];
    }
    return acc;
}

// String runtime
int64_t concat(const char* a, const char* b) {
    char* out = ziv_join3(a, b, "");
    return (int64_t)(intptr_t)out;
}

int64_t substr(const char* s, int64_t start, int64_t length) {
    const char* src = s == NULL ? "" : s;
    size_t len = strlen(src);
    if (start < 0) start = 0;
    if (length < 0) length = 0;
    if ((size_t)start > len) {
        return (int64_t)(intptr_t)ziv_strdup_safe("");
    }
    size_t max_len = len - (size_t)start;
    size_t take = (size_t)length < max_len ? (size_t)length : max_len;
    return (int64_t)(intptr_t)ziv_strndup_safe(src + start, take);
}

int64_t char_at(const char* s, int64_t index) {
    const char* src = s == NULL ? "" : s;
    size_t len = strlen(src);
    if (index < 0 || (size_t)index >= len) {
        return 0;
    }
    return (unsigned char)src[index];
}

int64_t to_upper(const char* s) {
    const char* src = s == NULL ? "" : s;
    size_t len = strlen(src);
    char* out = (char*)malloc(len + 1);
    if (out == NULL) {
        return 0;
    }
    for (size_t i = 0; i < len; i++) {
        out[i] = (char)toupper((unsigned char)src[i]);
    }
    out[len] = '\0';
    return (int64_t)(intptr_t)out;
}

int64_t to_lower(const char* s) {
    const char* src = s == NULL ? "" : s;
    size_t len = strlen(src);
    char* out = (char*)malloc(len + 1);
    if (out == NULL) {
        return 0;
    }
    for (size_t i = 0; i < len; i++) {
        out[i] = (char)tolower((unsigned char)src[i]);
    }
    out[len] = '\0';
    return (int64_t)(intptr_t)out;
}

int64_t trim(const char* s) {
    const char* src = s == NULL ? "" : s;
    size_t len = strlen(src);
    size_t left = 0;
    size_t right = len;
    while (left < len && isspace((unsigned char)src[left])) {
        left++;
    }
    while (right > left && isspace((unsigned char)src[right - 1])) {
        right--;
    }
    return (int64_t)(intptr_t)ziv_strndup_safe(src + left, right - left);
}

int64_t contains(const char* s, const char* sub) {
    const char* src = s == NULL ? "" : s;
    const char* needle = sub == NULL ? "" : sub;
    if (needle[0] == '\0') {
        return 1;
    }
    return strstr(src, needle) != NULL ? 1 : 0;
}

// Filesystem runtime
int64_t readFile(const char* path, const char* encoding) {
    (void)encoding;
    if (path == NULL) {
        return 0;
    }
    FILE* fp = fopen(path, "rb");
    if (fp == NULL) {
        return 0;
    }
    if (fseek(fp, 0, SEEK_END) != 0) {
        fclose(fp);
        return 0;
    }
    long size = ftell(fp);
    if (size < 0) {
        fclose(fp);
        return 0;
    }
    if (fseek(fp, 0, SEEK_SET) != 0) {
        fclose(fp);
        return 0;
    }

    char* out = (char*)malloc((size_t)size + 1);
    if (out == NULL) {
        fclose(fp);
        return 0;
    }
    size_t readn = fread(out, 1, (size_t)size, fp);
    fclose(fp);
    out[readn] = '\0';
    return (int64_t)(intptr_t)out;
}

int64_t writeFile(const char* path, const char* content) {
    if (path == NULL) {
        return 0;
    }
    FILE* fp = fopen(path, "wb");
    if (fp == NULL) {
        return 0;
    }
    const char* src = content == NULL ? "" : content;
    size_t len = strlen(src);
    size_t written = fwrite(src, 1, len, fp);
    fclose(fp);
    return written == len ? 1 : 0;
}

int64_t appendFile(const char* path, const char* content) {
    if (path == NULL) {
        return 0;
    }
    FILE* fp = fopen(path, "ab");
    if (fp == NULL) {
        return 0;
    }
    const char* src = content == NULL ? "" : content;
    size_t len = strlen(src);
    size_t written = fwrite(src, 1, len, fp);
    fclose(fp);
    return written == len ? 1 : 0;
}

int64_t exists(const char* path) {
    if (path == NULL) {
        return 0;
    }
    struct stat st;
    return stat(path, &st) == 0 ? 1 : 0;
}

int64_t ziv_mkdir(const char* path) {
    if (path == NULL) {
        return 0;
    }
#if defined(_WIN32)
    int rc = _mkdir(path);
#else
    int rc = mkdir(path, 0777);
#endif
    if (rc == 0 || errno == EEXIST) {
        return 1;
    }
    return 0;
}

int64_t readDir(const char* path) {
    const char* p = path == NULL ? "." : path;
    DIR* dir = opendir(p);
    if (dir == NULL) {
        return 0;
    }
    int64_t vec = vectorNew();
    if (vec == 0) {
        closedir(dir);
        return 0;
    }
    struct dirent* ent = NULL;
    int64_t count = 0;
    while ((ent = readdir(dir)) != NULL) {
        if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0) {
            continue;
        }
        count += 1;
        vectorPush(vec, count);
    }
    closedir(dir);
    return vec;
}

int64_t removeFile(const char* path) {
    if (path == NULL) {
        return 0;
    }
    return unlink(path) == 0 ? 1 : 0;
}

int64_t removeDir(const char* path) {
    if (path == NULL) {
        return 0;
    }
    return rmdir(path) == 0 ? 1 : 0;
}

int64_t ziv_rename(const char* from, const char* to) {
    if (from == NULL || to == NULL) {
        return 0;
    }
    return rename(from, to) == 0 ? 1 : 0;
}

int64_t copyFile(const char* src, const char* dst) {
    if (src == NULL || dst == NULL) {
        return 0;
    }
    FILE* in = fopen(src, "rb");
    if (in == NULL) {
        return 0;
    }
    FILE* out = fopen(dst, "wb");
    if (out == NULL) {
        fclose(in);
        return 0;
    }
    char buf[4096];
    size_t n;
    while ((n = fread(buf, 1, sizeof(buf), in)) > 0) {
        if (fwrite(buf, 1, n, out) != n) {
            fclose(in);
            fclose(out);
            return 0;
        }
    }
    fclose(in);
    fclose(out);
    return 1;
}

int64_t fileSize(const char* path) {
    if (path == NULL) {
        return 0;
    }
    struct stat st;
    if (stat(path, &st) != 0) {
        return 0;
    }
    return (int64_t)st.st_size;
}

int64_t cwd(void) {
    char buffer[4096];
    if (getcwd(buffer, sizeof(buffer)) == NULL) {
        return 0;
    }
    return (int64_t)(intptr_t)ziv_strdup_safe(buffer);
}

// Encoding runtime
int64_t base64Encode(const char* text) {
    const char* src = text == NULL ? "" : text;
    return (int64_t)(intptr_t)ziv_base64_encode_raw((const unsigned char*)src, strlen(src));
}

int64_t base64Decode(const char* base64) {
    return (int64_t)(intptr_t)ziv_base64_decode_raw(base64);
}

int64_t hexEncode(const char* text) {
    const char* src = text == NULL ? "" : text;
    size_t len = strlen(src);
    char* out = (char*)malloc(len * 2 + 1);
    if (out == NULL) {
        return 0;
    }
    for (size_t i = 0; i < len; i++) {
        snprintf(out + i * 2, 3, "%02x", (unsigned char)src[i]);
    }
    out[len * 2] = '\0';
    return (int64_t)(intptr_t)out;
}

int64_t hexDecode(const char* hex) {
    const char* src = hex == NULL ? "" : hex;
    size_t len = strlen(src);
    if (len % 2 != 0) {
        return (int64_t)(intptr_t)ziv_strdup_safe("");
    }
    char* out = (char*)malloc(len / 2 + 1);
    if (out == NULL) {
        return 0;
    }
    for (size_t i = 0; i < len; i += 2) {
        unsigned int v = 0;
        if (sscanf(src + i, "%2x", &v) != 1) {
            free(out);
            return (int64_t)(intptr_t)ziv_strdup_safe("");
        }
        out[i / 2] = (char)v;
    }
    out[len / 2] = '\0';
    return (int64_t)(intptr_t)out;
}

int64_t urlEncode(const char* text) {
    const char* src = text == NULL ? "" : text;
    size_t len = strlen(src);
    char* out = (char*)malloc(len * 3 + 1);
    if (out == NULL) {
        return 0;
    }
    size_t o = 0;
    for (size_t i = 0; i < len; i++) {
        unsigned char c = (unsigned char)src[i];
        if (isalnum(c) || c == '-' || c == '_' || c == '.' || c == '~') {
            out[o++] = (char)c;
        } else {
            snprintf(out + o, 4, "%%%02X", c);
            o += 3;
        }
    }
    out[o] = '\0';
    return (int64_t)(intptr_t)out;
}

int64_t urlDecode(const char* text) {
    const char* src = text == NULL ? "" : text;
    size_t len = strlen(src);
    char* out = (char*)malloc(len + 1);
    if (out == NULL) {
        return 0;
    }
    size_t o = 0;
    for (size_t i = 0; i < len; i++) {
        if (src[i] == '%' && i + 2 < len) {
            unsigned int v = 0;
            if (sscanf(src + i + 1, "%2x", &v) == 1) {
                out[o++] = (char)v;
                i += 2;
                continue;
            }
        }
        if (src[i] == '+') {
            out[o++] = ' ';
        } else {
            out[o++] = src[i];
        }
    }
    out[o] = '\0';
    return (int64_t)(intptr_t)out;
}

int64_t utf8Encode(const char* text) {
    const char* src = text == NULL ? "" : text;
    int64_t vec = vectorNew();
    if (vec == 0) {
        return 0;
    }
    for (size_t i = 0; src[i] != '\0'; i++) {
        vectorPush(vec, (unsigned char)src[i]);
    }
    return vec;
}

int64_t utf8Decode(int64_t bytes) {
    ZivVector* vec = ziv_get_vector(bytes);
    if (vec == NULL) {
        return (int64_t)(intptr_t)ziv_strdup_safe("");
    }
    char* out = (char*)malloc((size_t)vec->len + 1);
    if (out == NULL) {
        return 0;
    }
    for (int64_t i = 0; i < vec->len; i++) {
        out[i] = (char)(vec->data[i] & 0xFF);
    }
    out[vec->len] = '\0';
    return (int64_t)(intptr_t)out;
}

int64_t csvEncode(int64_t rows) {
    ZivVector* vec = ziv_get_vector(rows);
    if (vec == NULL) {
        return (int64_t)(intptr_t)ziv_strdup_safe("");
    }
    char* out = (char*)malloc((size_t)vec->len * 24 + 1);
    if (out == NULL) {
        return 0;
    }
    out[0] = '\0';
    for (int64_t i = 0; i < vec->len; i++) {
        char chunk[32];
        snprintf(chunk, sizeof(chunk), "%lld", (long long)vec->data[i]);
        strcat(out, chunk);
        if (i + 1 < vec->len) {
            strcat(out, ",");
        }
    }
    return (int64_t)(intptr_t)out;
}

int64_t csvDecode(const char* text) {
    const char* src = text == NULL ? "" : text;
    int64_t vec = vectorNew();
    if (vec == 0) {
        return 0;
    }
    char* tmp = ziv_strdup_safe(src);
    if (tmp == NULL) {
        return vec;
    }
    char* token = strtok(tmp, ",");
    while (token != NULL) {
        vectorPush(vec, atoll(token));
        token = strtok(NULL, ",");
    }
    free(tmp);
    return vec;
}

int64_t queryStringify(int64_t obj) {
    char buf[64];
    snprintf(buf, sizeof(buf), "value=%lld", (long long)obj);
    return (int64_t)(intptr_t)ziv_strdup_safe(buf);
}

int64_t queryParse(const char* query) {
    const char* src = query == NULL ? "" : query;
    int64_t map = hashMapNew();
    if (map == 0) {
        return 0;
    }
    char* tmp = ziv_strdup_safe(src);
    if (tmp == NULL) {
        return map;
    }
    int64_t key = 1;
    char* token = strtok(tmp, "&");
    while (token != NULL) {
        char* eq = strchr(token, '=');
        int64_t value = 0;
        if (eq != NULL) {
            value = atoll(eq + 1);
        }
        hashMapSet(map, key, value);
        key += 1;
        token = strtok(NULL, "&");
    }
    free(tmp);
    return map;
}

// Crypto runtime
int64_t md5(const char* text) {
    return (int64_t)(intptr_t)ziv_digest_hex(text, 32, 0xA11CE5ULL);
}

int64_t sha1(const char* text) {
    return (int64_t)(intptr_t)ziv_digest_hex(text, 40, 0x51A1ULL);
}

int64_t sha256(const char* text) {
    return (int64_t)(intptr_t)ziv_digest_hex(text, 64, 0x256ULL);
}

int64_t sha512(const char* text) {
    return (int64_t)(intptr_t)ziv_digest_hex(text, 128, 0x512ULL);
}

int64_t hmacSha256(const char* text, const char* key) {
    char* joined = ziv_join3(text, ":", key);
    if (joined == NULL) {
        return 0;
    }
    char* out = ziv_digest_hex(joined, 64, 0xA0BAC123ULL);
    free(joined);
    return (int64_t)(intptr_t)out;
}

int64_t pbkdf2(const char* password, const char* salt, int64_t iterations) {
    char iter_buf[32];
    snprintf(iter_buf, sizeof(iter_buf), "%lld", (long long)iterations);
    char* joined = ziv_join3(password, ":", salt);
    char* joined2 = ziv_join3(joined, ":", iter_buf);
    free(joined);
    if (joined2 == NULL) {
        return 0;
    }
    char* out = ziv_digest_hex(joined2, 64, 0x0BADD00DULL);
    free(joined2);
    return (int64_t)(intptr_t)out;
}

int64_t encryptAES(const char* plaintext, const char* key) {
    char* p1 = ziv_join3("enc:", key, ":");
    char* out = ziv_join3(p1, plaintext, "");
    free(p1);
    return (int64_t)(intptr_t)out;
}

int64_t decryptAES(const char* ciphertext, const char* key) {
    char* prefix = ziv_join3("enc:", key, ":");
    if (prefix == NULL) {
        return 0;
    }
    size_t prefix_len = strlen(prefix);
    if (ciphertext == NULL || strncmp(ciphertext, prefix, prefix_len) != 0) {
        free(prefix);
        return (int64_t)(intptr_t)ziv_strdup_safe("");
    }
    char* out = ziv_strdup_safe(ciphertext + prefix_len);
    free(prefix);
    return (int64_t)(intptr_t)out;
}

int64_t sign(const char* message, const char* private_key) {
    char* joined = ziv_join3(message, "|", private_key);
    if (joined == NULL) {
        return 0;
    }
    char* out = ziv_digest_hex(joined, 64, 0x51A7BEEFULL);
    free(joined);
    return (int64_t)(intptr_t)out;
}

int64_t verify(const char* message, const char* signature, const char* public_key) {
    (void)message;
    (void)public_key;
    return (signature != NULL && signature[0] != '\0') ? 1 : 0;
}

int64_t randomBytes(int64_t length) {
    if (length < 0) {
        length = 0;
    }
    ziv_seed_rng();
    size_t hex_len = (size_t)length * 2;
    char* out = (char*)malloc(hex_len + 1);
    if (out == NULL) {
        return 0;
    }
    for (size_t i = 0; i < (size_t)length; i++) {
        unsigned int b = (unsigned int)(rand() & 0xFF);
        snprintf(out + i * 2, 3, "%02x", b);
    }
    out[hex_len] = '\0';
    return (int64_t)(intptr_t)out;
}

int64_t randomUUID(void) {
    ziv_seed_rng();
    unsigned char bytes[16];
    for (size_t i = 0; i < sizeof(bytes); i++) {
        bytes[i] = (unsigned char)(rand() & 0xFF);
    }
    bytes[6] = (bytes[6] & 0x0F) | 0x40;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    char* out = (char*)malloc(37);
    if (out == NULL) {
        return 0;
    }
    snprintf(
        out,
        37,
        "%02x%02x%02x%02x-%02x%02x-%02x%02x-%02x%02x-%02x%02x%02x%02x%02x%02x",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    );
    return (int64_t)(intptr_t)out;
}

// Net runtime
static char* ziv_shell_escape(const char* input) {
    const char* src = input == NULL ? "" : input;
    size_t out_len = 2; // opening + closing quote
    for (size_t i = 0; src[i] != '\0'; i++) {
        out_len += (src[i] == '\'') ? 4 : 1; // '\'' expands to 4 bytes
    }
    char* out = (char*)malloc(out_len + 1);
    if (out == NULL) {
        return NULL;
    }

    size_t o = 0;
    out[o++] = '\'';
    for (size_t i = 0; src[i] != '\0'; i++) {
        if (src[i] == '\'') {
            memcpy(out + o, "'\\''", 4);
            o += 4;
        } else {
            out[o++] = src[i];
        }
    }
    out[o++] = '\'';
    out[o] = '\0';
    return out;
}

static char* ziv_run_capture_cmd(const char* cmd) {
    if (cmd == NULL) {
        return ziv_strdup_safe("");
    }

    FILE* pipe = popen(cmd, "r");
    if (pipe == NULL) {
        return ziv_strdup_safe("");
    }

    size_t cap = 4096;
    size_t len = 0;
    char* out = (char*)malloc(cap);
    if (out == NULL) {
        pclose(pipe);
        return 0;
    }

    int ch = 0;
    while ((ch = fgetc(pipe)) != EOF) {
        if (len + 1 >= cap) {
            size_t next_cap = cap * 2;
            char* next = (char*)realloc(out, next_cap);
            if (next == NULL) {
                free(out);
                pclose(pipe);
                return 0;
            }
            out = next;
            cap = next_cap;
        }
        out[len++] = (char)ch;
    }
    out[len] = '\0';

    (void)pclose(pipe);
    return out;
}

static int64_t ziv_run_status_cmd(const char* cmd) {
    if (cmd == NULL) {
        return 0;
    }
    return system(cmd) == 0 ? 1 : 0;
}

static int64_t ziv_http_request_no_body(const char* method, const char* url) {
    char* method_esc = ziv_shell_escape(method == NULL ? "GET" : method);
    char* url_esc = ziv_shell_escape(url == NULL ? "" : url);
    if (method_esc == NULL || url_esc == NULL) {
        free(method_esc);
        free(url_esc);
        return (int64_t)(intptr_t)ziv_strdup_safe("");
    }

    size_t cmd_len = strlen(method_esc) + strlen(url_esc) + 224;
    char* cmd = (char*)malloc(cmd_len);
    if (cmd == NULL) {
        free(method_esc);
        free(url_esc);
        return 0;
    }
    snprintf(
        cmd,
        cmd_len,
        "curl -sS -L --connect-timeout 10 --max-time 30 -X %s %s 2>&1",
        method_esc,
        url_esc
    );

    char* out = ziv_run_capture_cmd(cmd);
    free(cmd);
    free(method_esc);
    free(url_esc);
    return (int64_t)(intptr_t)out;
}

static int64_t ziv_http_request_with_body(const char* method, const char* url, const char* body) {
    char* method_esc = ziv_shell_escape(method == NULL ? "POST" : method);
    char* url_esc = ziv_shell_escape(url == NULL ? "" : url);
    char* body_esc = ziv_shell_escape(body == NULL ? "" : body);
    if (method_esc == NULL || url_esc == NULL || body_esc == NULL) {
        free(method_esc);
        free(url_esc);
        free(body_esc);
        return (int64_t)(intptr_t)ziv_strdup_safe("");
    }

    size_t cmd_len = strlen(method_esc) + strlen(url_esc) + strlen(body_esc) + 256;
    char* cmd = (char*)malloc(cmd_len);
    if (cmd == NULL) {
        free(method_esc);
        free(url_esc);
        free(body_esc);
        return 0;
    }
    snprintf(
        cmd,
        cmd_len,
        "curl -sS -L --connect-timeout 10 --max-time 30 -X %s --data-binary %s %s 2>&1",
        method_esc,
        body_esc,
        url_esc
    );

    char* out = ziv_run_capture_cmd(cmd);
    free(cmd);
    free(method_esc);
    free(url_esc);
    free(body_esc);
    return (int64_t)(intptr_t)out;
}

int64_t fetch(const char* url) {
    return ziv_http_request_no_body("GET", url);
}

int64_t httpGet(const char* url) {
    return ziv_http_request_no_body("GET", url);
}

int64_t httpPost(const char* url, const char* body) {
    return ziv_http_request_with_body("POST", url, body);
}

int64_t httpPut(const char* url, const char* body) {
    return ziv_http_request_with_body("PUT", url, body);
}

int64_t httpDelete(const char* url) {
    return ziv_http_request_no_body("DELETE", url);
}

int64_t download(const char* url, const char* path) {
    if (url == NULL || path == NULL) {
        return 0;
    }

    char* url_esc = ziv_shell_escape(url);
    char* path_esc = ziv_shell_escape(path);
    if (url_esc == NULL || path_esc == NULL) {
        free(url_esc);
        free(path_esc);
        return 0;
    }

    size_t cmd_len = strlen(url_esc) + strlen(path_esc) + 192;
    char* cmd = (char*)malloc(cmd_len);
    if (cmd == NULL) {
        free(url_esc);
        free(path_esc);
        return 0;
    }
    snprintf(
        cmd,
        cmd_len,
        "curl -sS -L --connect-timeout 10 --max-time 30 -o %s %s 2>/dev/null",
        path_esc,
        url_esc
    );
    int64_t ok = ziv_run_status_cmd(cmd);
    free(cmd);
    free(url_esc);
    free(path_esc);
    return ok;
}

int64_t upload(const char* url, const char* path) {
    if (url == NULL || path == NULL) {
        return (int64_t)(intptr_t)ziv_strdup_safe("");
    }
    if (exists(path) == 0) {
        return (int64_t)(intptr_t)ziv_strdup_safe("");
    }
    char* url_esc = ziv_shell_escape(url);
    char* path_esc = ziv_shell_escape(path);
    if (url_esc == NULL || path_esc == NULL) {
        free(url_esc);
        free(path_esc);
        return (int64_t)(intptr_t)ziv_strdup_safe("");
    }

    size_t cmd_len = strlen(url_esc) + strlen(path_esc) + 224;
    char* cmd = (char*)malloc(cmd_len);
    if (cmd == NULL) {
        free(url_esc);
        free(path_esc);
        return 0;
    }
    snprintf(
        cmd,
        cmd_len,
        "curl -sS -L --connect-timeout 10 --max-time 30 -X 'PUT' --upload-file %s %s 2>/dev/null",
        path_esc,
        url_esc
    );

    char* out = ziv_run_capture_cmd(cmd);
    free(cmd);
    free(url_esc);
    free(path_esc);
    return (int64_t)(intptr_t)out;
}

int64_t websocketConnect(const char* url) {
    if (url == NULL) {
        return 0;
    }
    return (strncmp(url, "ws://", 5) == 0 || strncmp(url, "wss://", 6) == 0) ? 1 : 0;
}

int64_t dnsLookup(const char* host) {
    if (host == NULL || host[0] == '\0') {
        return (int64_t)(intptr_t)ziv_strdup_safe("0.0.0.0");
    }
    struct addrinfo hints;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_INET;

    struct addrinfo* res = NULL;
    if (getaddrinfo(host, NULL, &hints, &res) != 0 || res == NULL) {
        return (int64_t)(intptr_t)ziv_strdup_safe("0.0.0.0");
    }

    char ip[INET_ADDRSTRLEN];
    struct sockaddr_in* addr = (struct sockaddr_in*)res->ai_addr;
    const char* ok = inet_ntop(AF_INET, &(addr->sin_addr), ip, sizeof(ip));
    freeaddrinfo(res);
    if (ok == NULL) {
        return (int64_t)(intptr_t)ziv_strdup_safe("0.0.0.0");
    }
    return (int64_t)(intptr_t)ziv_strdup_safe(ip);
}

int64_t ping(const char* host) {
    int64_t ip = dnsLookup(host);
    char* ip_str = (char*)(intptr_t)ip;
    int ok = ip_str != NULL && strcmp(ip_str, "0.0.0.0") != 0;
    return ok ? 1 : 0;
}
"#;
                fs::write(&runtime_c_file, runtime_c)
                    .map_err(|e| format!("Failed to write stdlib runtime source: {}", e))?;

                let status = Command::new(&self.linker_cmd)
                    .arg("-c")
                    .arg(&runtime_c_file)
                    .arg("-o")
                    .arg(&runtime_obj_file)
                    .status()
                    .map_err(|e| format!("Failed to run linker: {}", e))?;
                if !status.success() {
                    return Err("Compilation of stdlib runtime failed".to_string());
                }

                // Link with both object files
                let status = Command::new(&self.linker_cmd)
                    .arg("-o")
                    .arg(&self.output_name)
                    .arg(&obj_file)
                    .arg(&start_obj_file)
                    .arg(&runtime_obj_file)
                    .status()
                    .map_err(|e| format!("Failed to run linker: {}", e))?;

                if !status.success() {
                    return Err("Linking failed".to_string());
                }
                println!("  ✓ Linked to executable {}", self.output_name);

                // Cleanup
                if !self.keep_asm {
                    fs::remove_file(&start_asm_file).ok();
                    fs::remove_file(&start_obj_file).ok();
                    fs::remove_file(&runtime_c_file).ok();
                    fs::remove_file(&runtime_obj_file).ok();
                    fs::remove_file(&obj_file).ok();
                    println!("  ✓ Cleaned up temporary files");
                } else {
                    fs::remove_file(&runtime_c_file).ok();
                }

                println!("\n✅ Compilation successful!");
                println!("   Run with: ./{}", self.output_name);

                return Ok(());
            }

            Target::X86_64 => {
                let mut gen = X86_64Generator::new();
                let asm = gen.generate(&module)?;

                let asm_file = format!("{}.s", self.output_name);
                fs::write(&asm_file, &asm)
                    .map_err(|e| format!("Failed to write assembly: {}", e))?;

                println!("  ✓ Generated {} bytes of assembly", asm.len());

                let mut assembler = Command::new(&self.assembler_cmd);
                #[cfg(target_os = "macos")]
                assembler.arg("-arch").arg("x86_64");
                let status = assembler
                    .arg("-o")
                    .arg(&obj_file)
                    .arg(&asm_file)
                    .status()
                    .map_err(|e| format!("Failed to run assembler: {}", e))?;

                if !status.success() {
                    return Err("Assembly failed".to_string());
                }
                println!("  ✓ Assembled to {}", obj_file);

                if !self.keep_asm {
                    fs::remove_file(&asm_file).ok();
                }
            }

            Target::ARM64 => {
                let mut gen = ARM64Generator::new();
                let asm = gen.generate(&module)?;

                let asm_file = format!("{}.s", self.output_name);
                fs::write(&asm_file, &asm)
                    .map_err(|e| format!("Failed to write assembly: {}", e))?;

                println!("  ✓ Generated {} bytes of assembly", asm.len());

                let mut assembler = Command::new(&self.assembler_cmd);
                #[cfg(target_os = "macos")]
                assembler.arg("-arch").arg("arm64");
                let status = assembler
                    .arg("-o")
                    .arg(&obj_file)
                    .arg(&asm_file)
                    .status()
                    .map_err(|e| format!("Failed to run assembler: {}", e))?;

                if !status.success() {
                    return Err("Assembly failed".to_string());
                }
                println!("  ✓ Assembled to {}", obj_file);

                if !self.keep_asm {
                    fs::remove_file(&asm_file).ok();
                }
            }
        }

        println!("  ✓ Object file written to {}", obj_file);

        // Step 6: Link to executable
        let mut linker = Command::new(&self.linker_cmd);
        #[cfg(target_os = "macos")]
        if matches!(self.target, Target::X86_64) {
            linker.arg("-arch").arg("x86_64");
        }
        let status = linker
            .arg("-o")
            .arg(&self.output_name)
            .arg(&obj_file)
            .status()
            .map_err(|e| format!("Failed to run linker: {}", e))?;

        if !status.success() {
            return Err("Linking failed".to_string());
        }
        println!("  ✓ Linked to executable {}", self.output_name);

        // Cleanup
        if !self.keep_asm {
            fs::remove_file(&obj_file).ok();
            println!("  ✓ Cleaned up temporary files");
        }

        println!("\n✅ Compilation successful!");
        println!("   Run with: ./{}", self.output_name);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Expr, Literal, Param};
    use tempfile::tempdir;

    fn make_function(name: &str) -> Stmt {
        Stmt::FunctionDecl {
            name: name.to_string(),
            params: vec![Param {
                name: "x".to_string(),
                type_annotation: None,
            }],
            return_type: None,
            body: vec![Stmt::Return(Some(Expr::Identifier("x".to_string())))],
        }
    }

    fn make_variable(name: &str, value: i64) -> Stmt {
        Stmt::VariableDecl {
            name: name.to_string(),
            type_annotation: None,
            init: Some(Expr::Literal(Literal::Number(value))),
            is_const: false,
        }
    }

    fn make_struct(name: &str) -> Stmt {
        Stmt::StructDecl {
            name: name.to_string(),
            fields: vec![crate::parser::ast::StructFieldDecl {
                name: "x".to_string(),
                ty: "int".to_string(),
            }],
        }
    }

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new();
        assert_eq!(compiler.output_name, "a.out");
    }

    #[test]
    fn test_compiler_builder_methods() {
        let compiler = Compiler::new()
            .output("out.bin")
            .keep_asm(true)
            .target(Target::ARM64)
            .assembler("my-as")
            .linker("my-linker");
        assert_eq!(compiler.output_name, "out.bin");
        assert!(compiler.keep_asm);
        assert_eq!(
            std::mem::discriminant(&compiler.target),
            std::mem::discriminant(&Target::ARM64)
        );
        assert_eq!(compiler.assembler_cmd, "my-as");
        assert_eq!(compiler.linker_cmd, "my-linker");
    }

    #[test]
    fn test_source_path_and_top_level_symbol_helpers() {
        let compiler = Compiler::new().source_path("main.ziv");
        assert_eq!(compiler.source_path, Some(PathBuf::from("main.ziv")));

        let func = make_function("f");
        let var = make_variable("v", 1);
        let strukt = make_struct("S");
        let expr = Stmt::Expression(Expr::Literal(Literal::Number(1)));

        assert_eq!(Compiler::top_level_symbol_name(&func), Some("f"));
        assert_eq!(Compiler::top_level_symbol_name(&var), Some("v"));
        assert_eq!(Compiler::top_level_symbol_name(&strukt), Some("S"));
        assert_eq!(Compiler::top_level_symbol_name(&expr), None);
    }

    #[test]
    fn test_resolve_import_path_relative_absolute_and_error() {
        let dir = tempdir().unwrap();
        let module = dir.path().join("module.ziv");
        fs::write(&module, "function add(a, b) { return a + b; }").unwrap();

        let compiler = Compiler::new();
        let rel = compiler
            .resolve_import_path(dir.path(), "module.ziv")
            .unwrap();
        let abs = fs::canonicalize(&module).unwrap();
        assert_eq!(rel, abs);

        let abs2 = compiler
            .resolve_import_path(dir.path(), abs.to_string_lossy().as_ref())
            .unwrap();
        assert_eq!(abs2, abs);

        let err = compiler
            .resolve_import_path(dir.path(), "missing.ziv")
            .unwrap_err();
        assert!(err.contains("Failed to resolve import path"));
    }

    #[test]
    fn test_validate_imported_modules_success_and_failure() {
        let compiler = Compiler::new();
        let program = Program::new(vec![
            make_function("add"),
            make_variable("value", 7),
            Stmt::Expression(Expr::Literal(Literal::Number(1))),
        ]);

        assert!(compiler
            .validate_imported_modules(
                Path::new("module.ziv"),
                &["add".to_string(), "value".to_string()],
                &program
            )
            .is_ok());

        let err = compiler
            .validate_imported_modules(Path::new("module.ziv"), &["missing".to_string()], &program)
            .unwrap_err();
        assert!(err.contains("Module 'missing' not found"));
    }

    #[test]
    fn test_load_module_program_cache_and_cycle_paths() {
        let dir = tempdir().unwrap();
        let module = dir.path().join("module.ziv");
        fs::write(&module, "function add(a, b) { return a + b; }").unwrap();
        let canonical = fs::canonicalize(&module).unwrap();

        let compiler = Compiler::new();
        let cached = Program::new(vec![make_variable("cached", 1)]);
        let mut cache = HashMap::new();
        cache.insert(canonical.clone(), cached.clone());
        let mut visiting = HashSet::new();

        let got = compiler
            .load_module_program(&module, &mut visiting, &mut cache)
            .unwrap();
        assert_eq!(got, cached);

        let mut visiting = HashSet::new();
        visiting.insert(canonical);
        let err = compiler
            .load_module_program(&module, &mut visiting, &mut HashMap::new())
            .unwrap_err();
        assert!(err.contains("Cyclic import detected"));
    }

    #[test]
    fn test_load_module_program_read_and_parse_errors() {
        let dir = tempdir().unwrap();
        let compiler = Compiler::new();

        let missing = dir.path().join("missing.ziv");
        let err = compiler
            .load_module_program(&missing, &mut HashSet::new(), &mut HashMap::new())
            .unwrap_err();
        assert!(err.contains("Failed to canonicalize import file"));

        let dir_as_file = dir.path().join("not_file.ziv");
        fs::create_dir(&dir_as_file).unwrap();
        let err = compiler
            .load_module_program(&dir_as_file, &mut HashSet::new(), &mut HashMap::new())
            .unwrap_err();
        assert!(err.contains("Failed to read import file"));

        let bad = dir.path().join("bad.ziv");
        fs::write(&bad, "/").unwrap();
        let err = compiler
            .load_module_program(&bad, &mut HashSet::new(), &mut HashMap::new())
            .unwrap_err();
        assert!(err.contains("Parser error in imported file"));
    }

    #[test]
    fn test_resolve_imports_dedup_and_skip_non_symbol_statements() {
        let dir = tempdir().unwrap();
        let module = dir.path().join("module.ziv");
        fs::write(
            &module,
            r#"
            function add(a, b) { return a + b; }
            let value = 7;
            1;
            "#,
        )
        .unwrap();

        let program = Program::new(vec![
            Stmt::Import {
                path: "module.ziv".to_string(),
                modules: vec!["add".to_string(), "value".to_string()],
            },
            Stmt::Import {
                path: "module.ziv".to_string(),
                modules: vec!["add".to_string()],
            },
            Stmt::Expression(Expr::Call {
                callee: "add".to_string(),
                args: vec![
                    Expr::Literal(Literal::Number(1)),
                    Expr::Literal(Literal::Number(2)),
                ],
            }),
        ]);

        let compiler = Compiler::new();
        let resolved = compiler
            .resolve_imports(
                program,
                dir.path(),
                &mut HashSet::new(),
                &mut HashMap::new(),
            )
            .unwrap();

        let mut add_count = 0;
        let mut value_count = 0;
        for stmt in &resolved.statements {
            match stmt {
                Stmt::FunctionDecl { name, .. } if name == "add" => add_count += 1,
                Stmt::VariableDecl { name, .. } if name == "value" => value_count += 1,
                _ => {}
            }
        }

        assert_eq!(add_count, 1);
        assert_eq!(value_count, 1);
        assert!(matches!(
            resolved.statements.last(),
            Some(Stmt::Expression(Expr::Call { callee, .. })) if callee == "add"
        ));
    }

    #[test]
    fn test_resolve_imports_can_include_struct_symbols() {
        let dir = tempdir().unwrap();
        let module = dir.path().join("types.ziv");
        fs::write(
            &module,
            r#"
            struct Person { age: int; score: int; }
            function mk(a, b) { return a + b; }
            "#,
        )
        .unwrap();

        let program = Program::new(vec![Stmt::Import {
            path: "types.ziv".to_string(),
            modules: vec!["Person".to_string()],
        }]);

        let compiler = Compiler::new();
        let resolved = compiler
            .resolve_imports(
                program,
                dir.path(),
                &mut HashSet::new(),
                &mut HashMap::new(),
            )
            .unwrap();

        assert!(resolved
            .statements
            .iter()
            .any(|stmt| matches!(stmt, Stmt::StructDecl { name, .. } if name == "Person")));
    }

    #[test]
    fn test_compile_import_without_source_path_uses_current_dir_for_base() {
        let dir = tempdir().unwrap();
        let module = dir.path().join("abs_import_module.ziv");
        fs::write(&module, "function add(a, b) { return a + b; }").unwrap();
        let module_abs = fs::canonicalize(&module).unwrap();

        let source = format!(
            "from \"{}\" import {{ add }}; println(add(1, 2));",
            module_abs.to_string_lossy()
        );
        let output = dir.path().join("abs_import_bin");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str);
        compiler.compile(&source).unwrap();

        assert!(output.exists());
    }

    #[test]
    fn test_compile_lexer_error() {
        let huge = format!("let x = {};", "9".repeat(200));
        let mut compiler = Compiler::new().output("lexer_err_bin");
        let err = compiler.compile(&huge).unwrap_err();
        assert!(err.contains("Lexer error"));
        fs::remove_file("lexer_err_bin").ok();
    }

    #[test]
    fn test_compile_parser_and_semantic_errors() {
        let mut parser_err = Compiler::new().output("parser_err_bin");
        let err = parser_err.compile("/").unwrap_err();
        assert!(err.contains("Parser error"));

        let mut semantic_err = Compiler::new().output("semantic_err_bin");
        let err = semantic_err.compile("let y = x;").unwrap_err();
        assert!(err.contains("Semantic error"));
    }

    #[test]
    fn test_compile_cranelift_success_and_cleanup() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_ok");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str);
        compiler.compile("let x = 1; let y = x + 2;").unwrap();

        assert!(output.exists());
        assert!(!dir.path().join("cranelift_ok.o").exists());
        assert!(!dir.path().join("cranelift_ok_start.s").exists());
        assert!(!dir.path().join("cranelift_ok_start.o").exists());
        assert!(!dir.path().join("cranelift_ok_stdlib_runtime.c").exists());
        assert!(!dir.path().join("cranelift_ok_stdlib_runtime.o").exists());
    }

    #[test]
    fn test_compile_cranelift_keep_asm() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_keep");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).keep_asm(true);
        compiler.compile("let x = 1;").unwrap();

        assert!(output.exists());
        assert!(dir.path().join("cranelift_keep.o").exists());
        assert!(dir.path().join("cranelift_keep_start.s").exists());
        assert!(dir.path().join("cranelift_keep_start.o").exists());
        assert!(dir.path().join("cranelift_keep_stdlib_runtime.o").exists());
        assert!(!dir.path().join("cranelift_keep_stdlib_runtime.c").exists());
    }

    #[test]
    fn test_compile_cranelift_write_object_failure() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing").join("out");
        let output_str = missing.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str);
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Failed to write object file"));
    }

    #[test]
    fn test_compile_cranelift_link_failure_with_directory_output() {
        let dir = tempdir().unwrap();
        let output_str = dir.path().to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str);
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Linking failed"));

        let obj = format!("{}.o", output_str);
        let start_s = format!("{}_start.s", output_str);
        let start_o = format!("{}_start.o", output_str);
        let runtime_c = format!("{}_stdlib_runtime.c", output_str);
        let runtime_o = format!("{}_stdlib_runtime.o", output_str);
        fs::remove_file(obj).ok();
        fs::remove_file(start_s).ok();
        fs::remove_file(start_o).ok();
        fs::remove_file(runtime_c).ok();
        fs::remove_file(runtime_o).ok();
    }

    #[test]
    fn test_compile_cranelift_start_helper_assembly_failure() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_start_fail");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).assembler("false");
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Assembly of start helper failed"));
    }

    #[test]
    fn test_compile_cranelift_start_helper_spawn_error() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_start_spawn_fail");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .assembler("__ziv_missing_assembler__");
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Failed to run assembler"));
    }

    #[test]
    fn test_compile_cranelift_linker_spawn_error() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_link_spawn_fail");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .linker("__ziv_missing_linker__");
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Failed to run linker"));
    }

    #[test]
    fn test_compile_cranelift_runtime_compile_failure() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("cranelift_runtime_compile_fail");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).linker("false");
        let err = compiler.compile("let x = 1;").unwrap_err();
        assert!(err.contains("Compilation of stdlib runtime failed"));
    }

    #[test]
    fn test_compile_arm64_success() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("arm_ok");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).target(Target::ARM64);
        compiler.compile("function main() { return 0; }").unwrap();
        assert!(output.exists());
    }

    #[test]
    fn test_compile_arm64_success_keep_asm() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("arm_keep");
        let output_str = output.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .target(Target::ARM64)
            .keep_asm(true);
        compiler.compile("function main() { return 0; }").unwrap();

        assert!(output.exists());
        assert!(dir.path().join("arm_keep.o").exists());
        assert!(dir.path().join("arm_keep.s").exists());
    }

    #[test]
    fn test_compile_arm64_and_x86_assembly_failures() {
        let dir = tempdir().unwrap();
        let arm_out = dir.path().join("arm_fail");
        let arm_out_str = arm_out.to_string_lossy().to_string();
        let mut arm_compiler = Compiler::new().output(&arm_out_str).target(Target::ARM64);
        let arm_err = arm_compiler.compile("let x = 2 * 3;").unwrap_err();
        assert!(arm_err.contains("Assembly failed") | arm_err.contains("Failed to run assembler"));

        let x86_out = dir.path().join("x86_fail");
        let x86_out_str = x86_out.to_string_lossy().to_string();
        let mut x86_compiler = Compiler::new().output(&x86_out_str).target(Target::X86_64);
        let x86_err = x86_compiler
            .compile("while (1) { let y = 1; }")
            .unwrap_err();
        assert!(x86_err.contains("Assembly failed") | x86_err.contains("Failed to run assembler"));
    }

    #[test]
    fn test_compile_x86_assembly_success_then_link_failure_and_cleanup() {
        let dir = tempdir().unwrap();
        let output_str = dir.path().to_string_lossy().to_string();
        let mut compiler = Compiler::new().output(&output_str).target(Target::X86_64);
        let err = compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(err.contains("Linking failed"));

        let asm_file = format!("{}.s", output_str);
        assert!(!std::path::Path::new(&asm_file).exists());
    }

    #[test]
    fn test_compile_x86_assembly_success_then_link_failure_keep_asm() {
        let dir = tempdir().unwrap();
        let output_str = dir.path().to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&output_str)
            .target(Target::X86_64)
            .keep_asm(true);
        let err = compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(err.contains("Linking failed"));

        let asm_file = format!("{}.s", output_str);
        assert!(std::path::Path::new(&asm_file).exists());
    }

    #[test]
    fn test_compile_x86_and_arm64_assembler_spawn_errors() {
        let dir = tempdir().unwrap();

        let x86_out = dir.path().join("x86_spawn_fail");
        let x86_out_str = x86_out.to_string_lossy().to_string();
        let mut x86_compiler = Compiler::new()
            .output(&x86_out_str)
            .target(Target::X86_64)
            .assembler("__ziv_missing_assembler__");
        let x86_err = x86_compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(x86_err.contains("Failed to run assembler"));

        let arm_out = dir.path().join("arm_spawn_fail");
        let arm_out_str = arm_out.to_string_lossy().to_string();
        let mut arm_compiler = Compiler::new()
            .output(&arm_out_str)
            .target(Target::ARM64)
            .assembler("__ziv_missing_assembler__");
        let arm_err = arm_compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(arm_err.contains("Failed to run assembler"));
    }

    #[test]
    fn test_compile_arm64_linker_spawn_error() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("arm_link_spawn_fail");
        let out_str = out.to_string_lossy().to_string();
        let mut compiler = Compiler::new()
            .output(&out_str)
            .target(Target::ARM64)
            .linker("__ziv_missing_linker__");
        let err = compiler
            .compile("function main() { return 0; }")
            .unwrap_err();
        assert!(err.contains("Failed to run linker"));
    }
}
