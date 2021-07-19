use sqlx::mysql::MySqlPool;
use std::fs;
use std::io;
use std::env;
use std::path::{Path};

const SEEDS_FOLDER: &str = "seeds";

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?;
  // load_seeder(&pool).await?;
  println!("seeder complete");
  Ok(())
}

async fn load_seeder(pool: &MySqlPool) -> anyhow::Result<()> {
  let path = env::current_dir().unwrap();
  let format_migrate = format!("{}/{}", path.display(), SEEDS_FOLDER);
  let entries = read_dir(format_migrate);
  for e in entries {
    for i in e.iter() {
        let sqlfile = concat!("{}/{}", SEEDS_FOLDER, i);
        sqlx::query_file!(sqlfile).execute(pool).await?;
    }
  }
  Ok(())
}