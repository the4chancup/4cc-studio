//! Exposes the build target triple to the About popup's version block as `STUDIO_TARGET_TRIPLE`.
fn main() {
    println!(
        "cargo:rustc-env=STUDIO_TARGET_TRIPLE={}",
        std::env::var("TARGET").expect("TARGET is set by cargo")
    );
}
