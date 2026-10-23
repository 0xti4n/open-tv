use std::{fs, io, path::Path};

use anyhow::{Context, Result};
use directories::ProjectDirs;

pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("dev", "nyxtv", "nyx-tv")
}

pub fn migrate_legacy_data_dir() -> Result<()> {
    let Some(current_dirs) = project_dirs() else {
        return Ok(());
    };
    // This identifies the legacy project directory before the Nyx TV rename.
    let Some(legacy_dirs) = ProjectDirs::from("dev", "fredol", "open-tv") else {
        return Ok(());
    };

    let legacy_data = legacy_dirs.data_dir();
    if !legacy_data.exists() {
        return Ok(());
    }

    let current_data = current_dirs.data_dir();
    if !current_data.exists() || !current_data.join("db.sqlite").exists() {
        copy_dir_if_missing(legacy_data, current_data).with_context(|| {
            format!(
                "failed to copy legacy data from {} to {}",
                legacy_data.display(),
                current_data.display()
            )
        })?;
    }

    for (from, to) in [
        (legacy_dirs.config_dir(), current_dirs.config_dir()),
        (legacy_dirs.cache_dir(), current_dirs.cache_dir()),
    ] {
        if from != legacy_data && to != current_data && from.exists() {
            copy_dir_if_missing(from, to).with_context(|| {
                format!(
                    "failed to copy legacy data from {} to {}",
                    from.display(),
                    to.display()
                )
            })?;
        }
    }

    Ok(())
}

fn copy_dir_if_missing(from: &Path, to: &Path) -> io::Result<bool> {
    if from == to || to.join("db.sqlite").exists() || !from.exists() {
        return Ok(false);
    }

    copy_dir_contents(from, to)?;
    Ok(true)
}

fn copy_dir_contents(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let source = entry.path();
        let destination = to.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if destination.exists() && !destination.is_dir() {
                continue;
            }
            copy_dir_contents(&source, &destination)?;
        } else if !destination.exists() {
            fs::copy(source, destination)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::copy_dir_if_missing;
    use std::{fs, io};
    use tempfile::tempdir;

    #[test]
    fn copies_legacy_directory_when_target_is_missing() -> io::Result<()> {
        let temp = tempdir()?;
        let source = temp.path().join("legacy");
        let target = temp.path().join("new");
        fs::create_dir_all(source.join("nested"))?;
        fs::write(source.join("db.sqlite"), b"legacy database")?;
        fs::write(source.join("nested/settings.json"), b"legacy settings")?;

        assert!(copy_dir_if_missing(&source, &target)?);
        assert_eq!(fs::read(target.join("db.sqlite"))?, b"legacy database");
        assert_eq!(
            fs::read(target.join("nested/settings.json"))?,
            b"legacy settings"
        );
        Ok(())
    }

    #[test]
    fn skips_when_target_already_has_database() -> io::Result<()> {
        let temp = tempdir()?;
        let source = temp.path().join("legacy");
        let target = temp.path().join("new");
        fs::create_dir_all(&source)?;
        fs::create_dir_all(&target)?;
        fs::write(source.join("db.sqlite"), b"legacy database")?;
        fs::write(source.join("extra.txt"), b"legacy extra")?;
        fs::write(target.join("db.sqlite"), b"current database")?;

        assert!(!copy_dir_if_missing(&source, &target)?);
        assert_eq!(fs::read(target.join("db.sqlite"))?, b"current database");
        assert!(!target.join("extra.txt").exists());
        Ok(())
    }

    #[test]
    fn does_not_overwrite_existing_files() -> io::Result<()> {
        let temp = tempdir()?;
        let source = temp.path().join("legacy");
        let target = temp.path().join("new");
        fs::create_dir_all(&source)?;
        fs::create_dir_all(&target)?;
        fs::write(source.join("db.sqlite"), b"legacy database")?;
        fs::write(source.join("settings.json"), b"legacy settings")?;
        fs::write(target.join("settings.json"), b"current settings")?;

        assert!(copy_dir_if_missing(&source, &target)?);
        assert_eq!(fs::read(target.join("db.sqlite"))?, b"legacy database");
        assert_eq!(fs::read(target.join("settings.json"))?, b"current settings");
        Ok(())
    }
}
