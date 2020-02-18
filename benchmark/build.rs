fn main() {
 
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap() 
    );

    println!(
        "cargo:rustc-env=CARGO_MANIFEST_DIR={}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap() 
    );
}