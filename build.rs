use std::path::PathBuf;

fn main() {
    let homebrew_prefix = PathBuf::from("/opt/homebrew");

    // 基本的なライブラリパス
    println!(
        "cargo:rustc-link-search=native={}/lib",
        homebrew_prefix.display()
    );
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    println!("cargo:rustc-link-search=native=/usr/lib");

    // LuaJITのパス
    println!(
        "cargo:rustc-link-search=native={}/opt/luajit/lib",
        homebrew_prefix.display()
    );
    println!("cargo:rustc-link-lib=dylib=luajit-5.1");

    // 環境変数の設定
    println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=11.0");
    println!("cargo:rustc-env=CC=/usr/bin/cc");

    // 再ビルドのトリガー
    println!("cargo:rerun-if-changed=build.rs");
}
