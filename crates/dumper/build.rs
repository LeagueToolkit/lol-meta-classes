fn main() {
    let mut config = cmake::Config::new("stubs");
    let target = config.build_target("stubs");
    println!(
        "cargo:rustc-link-search=native={}",
        target.build().join("build").display()
    );

    println!("cargo:rustc-link-lib=static=stubs");
}
