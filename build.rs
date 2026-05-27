use std::env;
use std::io;
use std::process::Command;

fn main() -> io::Result<()> {
  let git_hash = Command::new("git")
    .args(["rev-parse", "--short", "HEAD"])
    .output()
    .map(|output| {
      String::from_utf8(output.stdout)
        .map(|s| "_".to_owned() + &s)
        .unwrap_or_default()
    })?;

  let pkg_ver = env::var("CARGO_PKG_VERSION").unwrap();
  let full_ver = format!("{pkg_ver}_{git_hash}");
  println!(
    "cargo:rustc-env=FULL_VERSION={full_ver}" // unique version
  );

  Ok(())
}
