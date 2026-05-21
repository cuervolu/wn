fn main() {
    let hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "desconocido".to_string());

    println!("cargo:rustc-env=GIT_HASH={hash}");

    println!("cargo:rerun-if-changed=../../.git/HEAD");
}
