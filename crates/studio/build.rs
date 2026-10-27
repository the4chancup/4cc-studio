//! Compiles the preview's icon into the executable on Windows: the file icon Explorer, a
//! shortcut and the unpacked bundle show before the program runs, which Windows reads only
//! from the exe's own resources (DECISIONS "The preview's icon and its build dependency").

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(windows)]
    windows_resources()?;
    Ok(())
}

#[cfg(windows)]
fn windows_resources() -> std::io::Result<()> {
    let icon = concat!(env!("CARGO_MANIFEST_DIR"), "/../../resources/icon/icon.ico");
    println!("cargo:rerun-if-changed={icon}");
    winresource::WindowsResource::new()
        .set_icon(icon)
        // The version block winresource writes names the package (`studio`) by default, and
        // Task Manager lists a running program by its file description; the name is spelled
        // here because a build script runs before studio_core's `app_name!` exists
        // (DECISIONS "The exe's version details carry the codename").
        .set("FileDescription", "Unlimited Tool Works")
        .set("ProductName", "Unlimited Tool Works")
        .compile()
}
