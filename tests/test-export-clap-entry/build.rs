fn main() {
    println!("cargo:rustc-link-lib=test_dummy_plugin");
    println!(
        "cargo:rustc-link-search=native=target/{}",
        std::env::var("PROFILE").unwrap()
    );
}
