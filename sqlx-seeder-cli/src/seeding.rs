use anyhow::{bail, Context};
use console::style;
use std::fs::{self, File};
use std::io::{Read, Write};

const SEEDS_FOLDER: &str = "seeds";

pub struct Seeding {
    pub name: String,
    pub sql: String,
}

pub fn add_file(name: &str) -> anyhow::Result<()> {
    use chrono::prelude::*;
    use std::path::PathBuf;

    fs::create_dir_all(SEEDS_FOLDER).context("Unable to create seeds directory")?;

    let dt = Utc::now();
    let mut file_name = dt.format("%Y-%m-%d_%H-%M-%S").to_string();
    file_name.push_str("_");
    file_name.push_str(name);
    file_name.push_str(".sql");

    let mut path = PathBuf::new();
    path.push(SEEDS_FOLDER);
    path.push(&file_name);

    let mut file = File::create(path).context("Failed to create file")?;
    file.write_all(b"-- Add seed script here")
        .context("Could not write to file")?;

    println!("Created seed: '{}'", file_name);
    Ok(())
}

pub async fn run() -> anyhow::Result<()> {
    let migrator = crate::migrator::get()?;

    if !migrator.can_migrate_database() {
        bail!(
            "Database seeds not supported for {}",
            migrator.database_type()
        );
    }

    migrator.create_seed_table().await?;

    let seeds = load_seeds()?;

    for mig in seeds.iter() {
        let mut tx = migrator.begin_seed().await?;

        if tx.check_if_applied(&mig.name).await? {
            println!("Already applied seed: '{}'", mig.name);
            continue;
        }
        println!("Applying seed: '{}'", mig.name);

        tx.execute_seed(&mig.sql)
            .await
            .with_context(|| format!("Failed to run seed {:?}", &mig.name))?;

        tx.save_applied_seed(&mig.name)
            .await
            .context("Failed to insert seed")?;

        tx.commit().await.context("Failed")?;
    }

    Ok(())
}

pub async fn list() -> anyhow::Result<()> {
    let migrator = crate::migrator::get()?;

    if !migrator.can_migrate_database() {
        bail!(
            "Database seeds not supported for {}",
            migrator.database_type()
        );
    }

    let file_seeds = load_seeds()?;

    if migrator
        .check_if_database_exists(&migrator.get_database_name()?)
        .await?
    {
        let applied_seeds = migrator.get_seeds().await.unwrap_or_else(|_| {
            println!("Could not retrive data from seed table");
            Vec::new()
        });

        let mut width = 0;
        for mig in file_seeds.iter() {
            width = std::cmp::max(width, mig.name.len());
        }
        for mig in file_seeds.iter() {
            let status = if applied_seeds
                .iter()
                .find(|&m| mig.name == *m)
                .is_some()
            {
                style("Applied").green()
            } else {
                style("Not Applied").yellow()
            };

            println!("{:width$}\t{}", mig.name, status, width = width);
        }

        let orphans = check_for_orphans(file_seeds, applied_seeds);

        if let Some(orphans) = orphans {
            println!("\nFound seeds applied in the database that does not have a corresponding seed file:");
            for name in orphans {
                println!("{:width$}\t{}", name, style("Orphan").red(), width = width);
            }
        }
    } else {
        println!("No database found, listing seeds");

        for mig in file_seeds {
            println!("{}", mig.name);
        }
    }

    Ok(())
}

fn load_seeds() -> anyhow::Result<Vec<Migration>> {
    let entries = fs::read_dir(&SEEDS_FOLDER).context("Could not find 'seeds' dir")?;

    let mut seeds = Vec::new();

    for e in entries {
        if let Ok(e) = e {
            if let Ok(meta) = e.metadata() {
                if !meta.is_file() {
                    continue;
                }

                if let Some(ext) = e.path().extension() {
                    if ext != "sql" {
                        println!("Wrong ext: {:?}", ext);
                        continue;
                    }
                } else {
                    continue;
                }

                let mut file = File::open(e.path())
                    .with_context(|| format!("Failed to open: '{:?}'", e.file_name()))?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .with_context(|| format!("Failed to read: '{:?}'", e.file_name()))?;

                seeds.push(Migration {
                    name: e.file_name().to_str().unwrap().to_string(),
                    sql: contents,
                });
            }
        }
    }

    seeds.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

    Ok(seeds)
}

fn check_for_orphans(
    file_seeds: Vec<Migration>,
    applied_seeds: Vec<String>,
) -> Option<Vec<String>> {
    let orphans: Vec<String> = applied_seeds
        .iter()
        .filter(|m| !file_seeds.iter().any(|fm| fm.name == **m))
        .cloned()
        .collect();

    if orphans.len() > 0 {
        Some(orphans)
    } else {
        None
    }
}
