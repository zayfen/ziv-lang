// 测试最简单的程序
let code = r#"
    let x = 42;
    let y = 10;
    let z = x + y;
"#;
    
    println!("\n✅ Code生成成功");
    println!("\n  LLVM IR:");
    println!("{}", module);
}

