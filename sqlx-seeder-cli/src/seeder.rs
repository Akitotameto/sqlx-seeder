use anyhow::{bail, Context};
use chrono::Utc;
use console::style;
use sqlx::{AnyConnection, Connection};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use crate::error::{BoxDynError, Error};

#[derive(Debug, Clone)]
pub struct Seeder {
    pub version: i64,
    pub description: Cow<'static, str>,
    pub migration_type: MigrationType,
    pub sql: Cow<'static, str>,
    pub checksum: Cow<'static, [u8]>,
}

#[derive(Debug, Clone)]
pub struct AppliedSeeder {
    pub version: i64,
    pub checksum: Cow<'static, [u8]>,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SeederError {
    #[error("while executing migrations: {0}")]
    Execute(#[from] Error),

    #[error("while resolving migrations: {0}")]
    Source(#[source] BoxDynError),

    #[error("migration {0} was previously applied but is missing in the resolved migrations")]
    VersionMissing(i64),

    #[error("migration {0} was previously applied but has been modified")]
    VersionMismatch(i64),

    #[error("cannot mix reversible migrations with simple migrations. All migrations should be reversible or simple migrations")]
    InvalidMixReversibleAndSimple,

    // NOTE: this will only happen with a database that does not have transactional DDL (.e.g, MySQL or Oracle)
    #[error(
        "migration {0} is partially applied; fix and remove row from `_sqlx_migrations` table"
    )]
    Dirty(i64),
}

fn validate_applied_migrations(
    applied_migrations: &[AppliedSeeder],
    migrator: &Seeder,
    ignore_missing: bool,
) -> Result<(), SeederError> {
    if ignore_missing {
        return Ok(());
    }

    let migrations: HashSet<_> = migrator.iter().map(|m| m.version).collect();

    for applied_migration in applied_migrations {
        if !migrations.contains(&applied_migration.version) {
            return Err(SeederError::VersionMissing(applied_migration.version));
        }
    }

    Ok(())
}

pub async fn run(
    migration_source: &str,
    uri: &str,
    dry_run: bool,
    ignore_missing: bool,
) -> anyhow::Result<()> {
    let migrator = Seeder::new(Path::new(migration_source)).await?;
    let mut conn = AnyConnection::connect(uri).await?;

    conn.ensure_migrations_table().await?;

    let version = conn.dirty_version().await?;
    if let Some(version) = version {
        bail!(SeederError::Dirty(version));
    }

    let applied_migrations = conn.list_applied_migrations().await?;
    validate_applied_migrations(&applied_migrations, &migrator, ignore_missing)?;

    let applied_migrations: HashMap<_, _> = applied_migrations
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    for migration in migrator.iter() {
        if migration.migration_type.is_down_migration() {
            // Skipping down migrations
            continue;
        }

        match applied_migrations.get(&migration.version) {
            Some(applied_migration) => {
                if migration.checksum != applied_migration.checksum {
                    bail!(SeederError::VersionMismatch(migration.version));
                }
            }
            None => {
                let elapsed = if dry_run {
                    Duration::new(0, 0)
                } else {
                    conn.apply(migration).await?
                };
                let text = if dry_run { "Can apply" } else { "Applied" };

                println!(
                    "{} {}/{} {} {}",
                    text,
                    style(migration.version).cyan(),
                    style(migration.migration_type.label()).green(),
                    migration.description,
                    style(format!("({:?})", elapsed)).dim()
                );
            }
        }
    }

    Ok(())
}