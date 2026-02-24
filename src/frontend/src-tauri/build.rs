fn main() {
    // In dev builds, create a placeholder service binary if it doesn't exist
    // so that Tauri's resource validation doesn't fail during `tauri dev`.
    // In release builds, the real binary is built by beforeBuildCommand.
    //
    // Note: We use the PROFILE env var (set by Cargo for build scripts) instead of
    // #[cfg(debug_assertions)] because Cargo.toml sets debug-assertions = false
    // in [profile.dev], which would make the cfg check always false.
    let profile = std::env::var("PROFILE").unwrap_or_default();
    if profile != "release" {
        let service_exe = std::path::Path::new("../../backend/target/release/netninja-service.exe");
        if !service_exe.exists() {
            if let Some(parent) = service_exe.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(service_exe, b"");
        }
    }

    tauri_build::build()
}
