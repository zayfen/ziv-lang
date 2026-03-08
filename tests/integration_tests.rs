use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ziv")
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
                    let resp_body =
                        if method == "GET" && (path == "/fetch" || path.starts_with("/get")) {
                            "FETCH_OK".to_string()
                        } else if method == "GET"
                            && (path == "/download" || path.starts_with("/bytes/12"))
                        {
                            "BYTES_12_DATA".to_string()
                        } else if method == "POST" && path == "/post" {
                            format!("POST:{body}")
                        } else if method == "PUT" && (path == "/put" || path == "/upload") {
                            format!("PUT:{body}")
                        } else if method == "DELETE" && path == "/delete" {
                            "DELETE_OK".to_string()
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

#[test]
fn test_cli_no_args() {
    let output = Command::new(bin()).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage:"));
}

#[test]
fn test_cli_o_requires_argument() {
    let output = Command::new(bin()).arg("-o").output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("-o requires an argument"));
}

#[test]
fn test_cli_no_source_file() {
    let output = Command::new(bin()).arg("--keep-asm").output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No source file specified"));
}

#[test]
fn test_cli_read_file_error() {
    let output = Command::new(bin())
        .arg("does_not_exist.ziv")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error reading file"));
}

#[test]
fn test_cli_compilation_error() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("bad.ziv");
    fs::write(&src, "let y = x;").unwrap();

    let output = Command::new(bin())
        .arg(&src)
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Compilation error"));
}

#[test]
fn test_cli_success_and_keep_asm() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("ok.ziv");
    fs::write(&src, "let x = 1;").unwrap();

    let output = Command::new(bin())
        .arg("--keep-asm")
        .arg(&src)
        .arg("-o")
        .arg("out_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(Path::new(&dir.path().join("out_bin")).exists());
    assert!(Path::new(&dir.path().join("out_bin.o")).exists());
    assert!(Path::new(&dir.path().join("out_bin_start.s")).exists());
    assert!(Path::new(&dir.path().join("out_bin_start.o")).exists());
}

#[test]
fn test_cli_compiled_program_emits_print_output() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("hello_print.ziv");
    fs::write(
        &src,
        r#"
        print("Hello, ");
        println("Ziv!");
        println(42);
        println(10 + 20);
        "#,
    )
    .unwrap();

    let output = Command::new(bin())
        .arg(&src)
        .arg("-o")
        .arg("hello_print_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("hello_print_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(stdout, "Hello, Ziv!\n42\n30\n");
}

#[test]
fn test_cli_from_import_compiles_and_runs() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main.ziv");
    let math = dir.path().join("math.ziv");

    fs::write(
        &main,
        r#"
        from { "./math.ziv" } import { add };
        println(add(20, 22));
        "#,
    )
    .unwrap();
    fs::write(
        &math,
        r#"
        function add(a, b) { return a + b; }
        function sub(a, b) { return a - b; }
        "#,
    )
    .unwrap();

    let output = Command::new(bin())
        .arg(&main)
        .arg("-o")
        .arg("import_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("import_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "42\n");
}

#[test]
fn test_cli_from_import_missing_module_reports_error() {
    let dir = tempdir().unwrap();
    let main = dir.path().join("main_missing_import.ziv");
    let math = dir.path().join("math_missing_import.ziv");

    fs::write(
        &main,
        r#"
        from { "./math_missing_import.ziv" } import { missing };
        println(missing(1, 2));
        "#,
    )
    .unwrap();
    fs::write(&math, "function add(a, b) { return a + b; }").unwrap();

    let output = Command::new(bin())
        .arg(&main)
        .arg("-o")
        .arg("missing_import_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Module 'missing' not found"));
}

#[test]
fn test_from_import_full_demo_example_compiles_and_runs() {
    let dir = tempdir().unwrap();
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("from_import")
        .join("full_demo.ziv");

    let output = Command::new(bin())
        .arg(&example)
        .arg("-o")
        .arg("from_import_demo_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("from_import_demo_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "27\n42\n9\n7\n");
}

#[test]
fn test_cli_struct_example_compiles_and_runs() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("struct_demo.ziv");
    fs::write(
        &src,
        r#"
        struct Person {
            age: int;
            score: int;
        }

        let p: Person = Person.(age = 18, score = 90);
        println(p.age);
        p += Person.(age = 20);
        println(p.age);
        println(p.score);
        "#,
    )
    .unwrap();

    let output = Command::new(bin())
        .arg(&src)
        .arg("-o")
        .arg("struct_demo_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("struct_demo_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "18\n20\n90\n");
}

#[test]
fn test_struct_demo_example_compiles_and_runs() {
    let dir = tempdir().unwrap();
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("struct")
        .join("struct_demo.ziv");

    let output = Command::new(bin())
        .arg(&example)
        .arg("-o")
        .arg("struct_demo_example_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("struct_demo_example_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "18\n20\n90\n");
}

#[test]
fn test_module_style_example_runs_with_zero_exit_code() {
    let dir = tempdir().unwrap();
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("from_import")
        .join("advanced_ops.ziv");

    let output = Command::new(bin())
        .arg(&example)
        .arg("-o")
        .arg("advanced_ops_module_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("advanced_ops_module_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "");
}

#[test]
fn test_struct_function_param_return_example_compiles_and_runs() {
    let dir = tempdir().unwrap();
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("struct")
        .join("struct_func_demo.ziv");

    let output = Command::new(bin())
        .arg(&example)
        .arg("-o")
        .arg("struct_func_demo_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("struct_func_demo_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "18\n18\n");
}

#[test]
fn test_function_argument_example_compiles_and_runs() {
    let dir = tempdir().unwrap();
    let example = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("function")
        .join("function_arg_demo.ziv");

    let output = Command::new(bin())
        .arg(&example)
        .arg("-o")
        .arg("function_arg_demo_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("function_arg_demo_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "42\n43\n");
}

#[test]
fn test_container_runtime_behaviour_compiles_and_runs() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("container_runtime.ziv");
    fs::write(
        &src,
        r#"
        let v = vectorNew();
        vectorPush(v, 7);
        vectorPush(v, 9);
        println(vectorLen(v));
        println(vectorGet(v, 0));
        println(vectorContains(v, 9));

        let m = hashMapNew();
        hashMapSet(m, 1, 100);
        hashMapSet(m, 2, 200);
        println(hashMapLen(m));
        println(hashMapGet(m, 2));
        println(hashMapHas(m, 3));
        "#,
    )
    .unwrap();

    let output = Command::new(bin())
        .arg(&src)
        .arg("-o")
        .arg("container_runtime_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("container_runtime_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "2\n7\n1\n2\n200\n0\n");
}

#[test]
fn test_stdlib_runtime_modules_behaviour_compiles_and_runs() {
    let (base_url, stop_server, server_handle) = start_mock_http_server();
    let fetch_url = format!("{base_url}/fetch");
    let post_url = format!("{base_url}/post");
    let put_url = format!("{base_url}/put");
    let delete_url = format!("{base_url}/delete");
    let download_url = format!("{base_url}/download");
    let upload_url = format!("{base_url}/upload");

    let dir = tempdir().unwrap();
    let src = dir.path().join("stdlib_runtime_modules.ziv");
    let source = format!(
        r#"
        println(strlen(concat("ab", "cd")));
        println(strlen(substr("abcdef", 1, 3)));
        println(char_at(to_upper("az"), 0));
        println(contains(trim("  hi  "), "hi"));

        println(writeFile("fs_a.txt", "abc"));
        println(exists("fs_a.txt"));
        println(fileSize("fs_a.txt"));
        println(appendFile("fs_a.txt", "de"));
        println(fileSize("fs_a.txt"));
        println(copyFile("fs_a.txt", "fs_b.txt"));
        println(fileSize("fs_b.txt"));
        println(rename("fs_b.txt", "fs_c.txt"));
        println(exists("fs_c.txt"));
        println(removeFile("fs_c.txt"));
        println(exists("fs_c.txt"));
        println(mkdir("fs_dir"));
        println(exists("fs_dir"));
        println(removeDir("fs_dir"));
        println(exists("fs_dir"));
        println(strlen(readFile("fs_a.txt", "utf-8")));
        println(mkdir("rd"));
        println(vectorLen(readDir("rd")));
        println(removeDir("rd"));

        println(strlen(base64Encode("ab")));
        println(strlen(base64Decode("YWI=")));
        println(strlen(hexEncode("ab")));
        println(strlen(hexDecode("6162")));
        println(strlen(urlEncode("a b")));
        println(strlen(urlDecode("a%20b")));
        let bytes = utf8Encode("AZ");
        println(vectorLen(bytes));
        println(strlen(utf8Decode(bytes)));
        println(strlen(csvEncode(bytes)));
        println(vectorLen(csvDecode("1,2,3")));
        println(strlen(queryStringify(12)));
        println(hashMapLen(queryParse("a=1&b=2")));

        println(strlen(md5("abc")));
        println(strlen(sha1("abc")));
        println(strlen(sha256("abc")));
        println(strlen(sha512("abc")));
        println(strlen(hmacSha256("abc", "k")));
        println(strlen(pbkdf2("p", "s", 10)));
        let encrypted = encryptAES("hello", "k");
        println(strlen(encrypted));
        println(strlen(decryptAES(encrypted, "k")));
        println(verify("m", sign("m", "k"), "pub"));
        println(strlen(randomBytes(8)));
        println(strlen(randomUUID()));

        println(strlen(fetch("{fetch_url}")));
        println(strlen(httpGet("{fetch_url}")));
        println(strlen(httpPost("{post_url}", "x")));
        println(strlen(httpPut("{put_url}", "x")));
        println(strlen(httpDelete("{delete_url}")));
        println(download("{download_url}", "net.txt"));
        println(exists("net.txt"));
        println(strlen(upload("{upload_url}", "net.txt")));
        println(websocketConnect("wss://example.com/ws"));
        println(strlen(dnsLookup("localhost")));
        println(ping("localhost"));
        "#
    );
    fs::write(&src, source).unwrap();

    let output = Command::new(bin())
        .arg(&src)
        .arg("-o")
        .arg("stdlib_runtime_modules_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("stdlib_runtime_modules_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    stop_server.store(true, Ordering::Relaxed);
    let _ = server_handle.join();
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "4\n3\n65\n1\n1\n1\n3\n1\n5\n1\n5\n1\n1\n1\n0\n1\n1\n1\n0\n5\n1\n0\n1\n4\n2\n4\n2\n5\n3\n2\n2\n5\n3\n8\n2\n32\n40\n64\n128\n64\n64\n11\n5\n1\n16\n36\n8\n8\n6\n5\n9\n1\n1\n17\n1\n9\n1\n"
    );
}

#[test]
fn test_println_can_output_fetch_and_httpget_string_results() {
    let (base_url, stop_server, server_handle) = start_mock_http_server();
    let fetch_url = format!("{base_url}/fetch");

    let dir = tempdir().unwrap();
    let src = dir.path().join("print_fetch_httpget.ziv");
    let source = format!(
        r#"
        println(fetch("{fetch_url}"));
        let resp = httpGet("{fetch_url}");
        println(resp);
        "#
    );
    fs::write(&src, source).unwrap();

    let output = Command::new(bin())
        .arg(&src)
        .arg("-o")
        .arg("print_fetch_httpget_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("print_fetch_httpget_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    stop_server.store(true, Ordering::Relaxed);
    let _ = server_handle.join();
    assert_eq!(String::from_utf8_lossy(&run.stdout), "FETCH_OK\nFETCH_OK\n");
}

#[test]
fn test_stdlib_math_array_utils_io_runtime_behaviour_compiles_and_runs() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("stdlib_math_array_utils_io_runtime.ziv");
    fs::write(
        &src,
        r#"
        println(abs(-5));
        let arr = push(0, 1);
        push(arr, 2);
        push(arr, 3);
        println(arrlen(arr));
        println(first(arr));
        println(last(arr));
        reverse(arr);
        println(get(arr, 0));
        set(arr, 1, 9);
        println(get(arr, 1));
        println(pop(arr));
        println(arrlen(arr));

        println(parseInt("2a", 16));
        println(parseFloat("7.9"));
        println(isNaN(1));
        println(isFinite(1));
        println(Number(8));
        println(strlen(String(1234)));
        println(Boolean(0));
        println(jsonParse("{\"v\":17}"));
        println(strlen(jsonStringify(77)));
        println(includes("abc", "bc"));
        println(indexOf("abc", "c"));
        println(startsWith("abc", "a"));
        println(endsWith("abc", "c"));
        let parts = split("x,yy", ",");
        println(vectorLen(parts));
        println(vectorGet(parts, 0));
        println(vectorGet(parts, 1));
        println(strlen(replace("abc", "b", "zz")));
        let mapped = map(arr, 0);
        println(arrlen(mapped));
        println(get(mapped, 1));
        let filtered = filter(arr, 0);
        println(arrlen(filtered));
        println(reduce(arr, 0, 10));

        eprint("E");
        eprintln("R");
        println(strlen(read()));
        println(strlen(input("P> ")));
        println(strlen(readAll()));
        printf("Q:%lld\n", 5);
        println(flush());
        "#,
    )
    .unwrap();

    let output = Command::new(bin())
        .arg(&src)
        .arg("-o")
        .arg("stdlib_math_array_utils_io_runtime_bin")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compile stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new(dir.path().join("stdlib_math_array_utils_io_runtime_bin"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "run stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "5\n3\n1\n3\n3\n9\n1\n2\n42\n7\n0\n1\n8\n4\n0\n17\n2\n1\n2\n1\n1\n2\n1\n2\n4\n2\n10\n2\n22\n0\nP> 0\n0\nQ:5\n1\n"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "ER\n");
}
