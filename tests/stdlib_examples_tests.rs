use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tempfile::tempdir;
use ziv::parser::ast::{Expr, Program, Stmt};
use ziv::parser::Parser;
use ziv::stdlib::Stdlib;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ziv")
}

fn stdlib_examples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/stdlib")
}

fn stdlib_example_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(stdlib_examples_dir())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("ziv"))
        .collect();
    files.sort();
    files
}

fn parse_http_request(stream: &mut TcpStream) -> (String, String, String) {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let mut data = Vec::new();
    let mut buf = [0_u8; 4096];

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                data.extend_from_slice(&buf[..n]);
                if n < buf.len() {
                    break;
                }
                if data.len() > 128 * 1024 {
                    break;
                }
            }
            Err(err)
                if err.kind() == std::io::ErrorKind::WouldBlock
                    || err.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(_) => break,
        }
    }

    let text = String::from_utf8_lossy(&data);
    let (header_text, body_text) = text
        .split_once("\r\n\r\n")
        .map(|(h, b)| (h, b))
        .unwrap_or((text.as_ref(), ""));

    let mut lines = header_text.lines();
    let request_line = lines.next().unwrap_or("GET / HTTP/1.1");
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or("GET").to_string();
    let path = request_parts.next().unwrap_or("/").to_string();

    (method, path, body_text.to_string())
}

fn write_http_response(stream: &mut TcpStream, body: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn start_mock_http_server() -> (String, Arc<AtomicBool>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();

    let stop = Arc::new(AtomicBool::new(false));
    let stop_for_thread = Arc::clone(&stop);

    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(30);
        while !stop_for_thread.load(Ordering::Relaxed) && Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let (method, path, body) = parse_http_request(&mut stream);
                    let resp_body = if method == "GET" && path.starts_with("/get") {
                        "FETCH_OK".to_string()
                    } else if method == "POST" && path == "/post" {
                        format!("POST:{body}")
                    } else if method == "PUT" && path == "/put" {
                        format!("PUT:{body}")
                    } else if method == "DELETE" && path == "/delete" {
                        "DELETE_OK".to_string()
                    } else if method == "GET" && path.starts_with("/bytes/12") {
                        "BYTES_12_DATA".to_string()
                    } else {
                        "UNKNOWN".to_string()
                    };
                    write_http_response(&mut stream, &resp_body);
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });

    (format!("http://127.0.0.1:{}", addr.port()), stop, handle)
}

fn expected_outputs(stem: &str) -> (&'static str, &'static str) {
    match stem {
        "hello" => ("Hello, Ziv!\n42\n30\n", ""),
        "io_demo" => ("io demo\n12\n0\nprompt> 0\n0\nfmt:10\n1\nio done\n", "err line\n"),
        "math_demo" => ("math demo\n10\n3\n7\n9\n1024\n5\n6\n7\nmath done\n", ""),
        "string_demo" => ("string demo\n3\n2\n3\n98\n65\n122\n1\n1\nstring done\n", ""),
        "array_demo" => ("array demo\n3\n10\n30\n20\n99\n30\n2\n99\n10\narray done\n", ""),
        "container_demo" => (
            "container demo\n0\n3\n15\n1\n10\n99\n20\n1\n0\n2\n100\n1\n200\n0\n300\n2\n2\n0\ncontainer done\n",
            "",
        ),
        "utils_demo" => (
            "utils demo\n42\n3\n0\n1\n12\n3\n0\n7\n2\n1\n1\n1\n1\n3\n2\n3\n1\n5\n3\n0\n4\n2\n14\nutils done\n",
            "",
        ),
        "filesystem_demo" => (
            "filesystem demo\n1\n1\n3\n1\n5\n5\n1\n5\n1\n1\n1\n0\n1\n0\n1\n1\n0\n1\n1\nfilesystem done\n",
            "",
        ),
        "net_demo" => (
            "net demo\n8\nFETCH_OK\nFETCH_OK\nFETCH_OK\nPOST:x=1\nPUT:y=2\nDELETE_OK\n1\n1\n1\nPUT:BYTES_12_DATA\n1\n1\n1\n1\nnet done\n",
            "",
        ),
        "crypto_demo" => ("crypto demo\n32\n40\n64\n128\n64\n64\n16\n5\n64\n1\n32\n36\ncrypto done\n", ""),
        "encoding_demo" => ("encoding demo\n4\n3\n6\n3\n5\n3\n3\n3\n8\n3\n8\n2\nencoding done\n", ""),
        _ => ("", ""),
    }
}

fn parse_program(source: &str) -> Program {
    let mut parser = Parser::new(source);
    parser.parse().unwrap()
}

fn collect_calls_in_expr(expr: &Expr, calls: &mut HashSet<String>) {
    match expr {
        Expr::Call { callee, args } => {
            calls.insert(callee.clone());
            for arg in args {
                collect_calls_in_expr(arg, calls);
            }
        }
        Expr::StructInit { fields, .. } => {
            for field in fields {
                collect_calls_in_expr(&field.value, calls);
            }
        }
        Expr::FieldAccess { object, .. } => collect_calls_in_expr(object, calls),
        Expr::Binary { left, right, .. } => {
            collect_calls_in_expr(left, calls);
            collect_calls_in_expr(right, calls);
        }
        Expr::Literal(_) | Expr::Identifier(_) => {}
    }
}

fn collect_calls_in_stmt(stmt: &Stmt, calls: &mut HashSet<String>) {
    match stmt {
        Stmt::Import { .. } => {}
        Stmt::StructDecl { .. } => {}
        Stmt::Expression(expr) => collect_calls_in_expr(expr, calls),
        Stmt::VariableDecl { init, .. } => {
            if let Some(expr) = init {
                collect_calls_in_expr(expr, calls);
            }
        }
        Stmt::Assignment { value, .. } => collect_calls_in_expr(value, calls),
        Stmt::StructMergeAssign { value, .. } => collect_calls_in_expr(value, calls),
        Stmt::FunctionDecl { body, .. } => {
            for stmt in body {
                collect_calls_in_stmt(stmt, calls);
            }
        }
        Stmt::Return(value) => {
            if let Some(expr) = value {
                collect_calls_in_expr(expr, calls);
            }
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_calls_in_expr(condition, calls);
            for stmt in then_branch {
                collect_calls_in_stmt(stmt, calls);
            }
            if let Some(else_branch) = else_branch {
                for stmt in else_branch {
                    collect_calls_in_stmt(stmt, calls);
                }
            }
        }
        Stmt::While { condition, body } => {
            collect_calls_in_expr(condition, calls);
            for stmt in body {
                collect_calls_in_stmt(stmt, calls);
            }
        }
        Stmt::Block(stmts) => {
            for stmt in stmts {
                collect_calls_in_stmt(stmt, calls);
            }
        }
    }
}

#[test]
fn test_stdlib_examples_cover_all_registered_functions() {
    let files = stdlib_example_files();
    assert!(
        !files.is_empty(),
        "expected examples in {}, found none",
        stdlib_examples_dir().display()
    );

    let mut calls = HashSet::new();
    for file in files {
        let source = fs::read_to_string(&file).unwrap();
        let program = parse_program(&source);
        for stmt in &program.statements {
            collect_calls_in_stmt(stmt, &mut calls);
        }
    }

    let stdlib = Stdlib::new();
    let mut missing: Vec<String> = stdlib
        .all_functions()
        .into_iter()
        .map(|func| func.name.clone())
        .filter(|name| !calls.contains(name))
        .collect();
    missing.sort();

    assert!(
        missing.is_empty(),
        "stdlib functions missing from examples: {}",
        missing.join(", ")
    );
}

#[test]
fn test_stdlib_examples_compile_and_run() {
    let files = stdlib_example_files();
    let dir = tempdir().unwrap();
    let (mock_base_url, stop_server, server_handle) = start_mock_http_server();

    for file in files {
        let stem = file.file_stem().unwrap().to_string_lossy().to_string();
        let output = dir.path().join(format!("{stem}_bin"));
        let source_path = if stem == "net_demo" {
            let source = fs::read_to_string(&file).unwrap();
            let patched = source
                .replace("https://httpbin.org", &mock_base_url)
                .replace(
                    "https://www.baidu.com",
                    &format!("{mock_base_url}/get?demo=baidu"),
                );
            let patched_file = dir.path().join("net_demo_local.ziv");
            fs::write(&patched_file, patched).unwrap();
            patched_file
        } else {
            file.clone()
        };

        let compile = Command::new(bin())
            .arg(&source_path)
            .arg("-o")
            .arg(&output)
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            compile.status.success(),
            "failed to compile {}\nstdout:\n{}\nstderr:\n{}",
            source_path.display(),
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        assert!(
            output.exists(),
            "missing output binary for {}",
            source_path.display()
        );

        let run = Command::new(&output)
            .current_dir(dir.path())
            .output()
            .unwrap();

        assert!(
            run.status.success(),
            "failed to run {}\nstdout:\n{}\nstderr:\n{}",
            output.display(),
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let (expected_stdout, expected_stderr) = expected_outputs(&stem);
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected_stdout,
            "unexpected stdout for {}",
            file.display()
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stderr),
            expected_stderr,
            "unexpected stderr for {}",
            file.display()
        );
    }

    stop_server.store(true, Ordering::Relaxed);
    let _ = server_handle.join();
}
