// use rust_seeder::lib_hello;

// fn main() {
//     lib_hello();
// }

use std::fs;
use std::io;
use std::env;
use std::path::{Path};
use std::process::Command;

fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
  Ok(fs::read_dir(path)?
      .filter_map(|entry| {
          let entry = entry.ok()?;
          if entry.file_type().ok()?.is_file() {
            println!("{}",entry.file_name().to_string_lossy().into_owned());
            Some(entry.file_name().to_string_lossy().into_owned())
          } else {
              None
          }
      })
      .collect())
}

fn main() {
  let path = env::current_dir().unwrap();
  println!("starting dir: {:?}", path.display());
  let format_migrate = format!("{}/seeds", path.display());
  read_dir(format_migrate);
}
