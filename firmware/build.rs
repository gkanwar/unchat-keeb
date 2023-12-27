use std::env;
use std::fs;
use std::process;
use std::io::{Result, Error, ErrorKind};

fn make_err(e: &str) -> Error {
  return Error::new(ErrorKind::Other, e);
}

// keymap files are formatted as for ZMK, i.e. in DeviceTree Source (DTS) format
// with possible C preprocessor directives
fn preprocess_dts(path: &str, out_dir: &str) -> Result<()> {
  let output = process::Command::new("gcc")
    .arg("-E")
    .arg("-Isrc/lib/zmk-dt-bindings")
    .arg("-Isrc/lib/zmk-dt-bindings/dt")
    .arg(path)
    .output()?;
  println!("output {:?}", output);
  if !output.status.success() {
    return Result::Err(make_err("preprocessor failed"));
  }

  return Result::Ok(());
}

fn main() -> Result<()> {
  println!("cargo:rerun-if-changed=src/lib/zmk-dt-bindings/");
  println!("cargo:rerun-if-changed=src/lib/keymaps/");

  let out_dir = env::var("OUT_DIR")
    .map_err(|e| make_err("OUT_DIR not set"))?;
  
  let entries = fs::read_dir("src/lib/keymaps/")
    .map_err(|e| make_err("read keymaps dir failed"))?;
  for dir in entries {
    let path = dir?.path();
    
    if !path.to_str().ok_or(make_err("hi"))?.ends_with(".keymap") {
      continue;
    }
    preprocess_dts(path.to_str().ok_or(make_err("hi"))?, &out_dir[..])?;
  }

  return Result::Ok(());
}
