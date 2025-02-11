fn main() {
    println!("cargo:rustc-link-search=/opt/homebrew/opt/llvm@18/lib");
    println!("cargo:rustc-link-lib=zstd");

    // 添加 LLVM 链接配置
    println!("cargo:rustc-link-lib=LLVM-18");

    // 如果需要，可以添加更多 LLVM 相关的库
    println!("cargo:rustc-link-lib=dylib=LLVM");
    println!("cargo:rustc-link-lib=dylib=LLVMSupport");
}
