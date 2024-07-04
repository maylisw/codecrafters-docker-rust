use anyhow::{Context, Result};
use std::{fs, os::unix, path::PathBuf};

pub struct IsolatedFileSystem {
    pub root_dir: PathBuf,
}

impl IsolatedFileSystem {
    pub fn setup(command: &String) -> Result<IsolatedFileSystem> {
        let root_dir = PathBuf::from("./sandbox");
        fs::create_dir_all(&root_dir)
            .with_context(|| format!("Failed to create '{:#?}' sandbox directory", root_dir))?;

        let dev = "dev";
        fs::create_dir_all(root_dir.join(dev)).with_context(|| {
            format!("Failed to create '{:#?}' directory", root_dir.join(dev))
        })?;
        fs::write("/dev/null", b"")
            .with_context(|| format!("Failed to create '/dev/null' file"))?;

        let command_path = root_dir.join(
            PathBuf::from(command)
                .parent()
                .unwrap()
                .strip_prefix("/")
                .with_context(|| {
                    format!(
                        "Failed to strip '/' prefix from {:#?}",
                        PathBuf::from(command).parent().unwrap(),
                    )
                })?,
        );

        fs::create_dir_all(&command_path)
            .with_context(|| format!("Failed to create directories for cmd {:#?}", command_path))?;

        fs::copy(
            command,
            root_dir.join(PathBuf::from(command).strip_prefix("/")?),
        )
        .with_context(|| format!("Failed to copy '{}'", command))?;

        return Ok(IsolatedFileSystem {
            root_dir: root_dir,
        });
    }

    pub fn chroot(&self) -> Result<(), anyhow::Error> {
        unix::fs::chroot(&self.root_dir).with_context(|| {
            format!(
                "Failed to chroot '{:#?}' sandbox directory",
                &self.root_dir
            )
        })?;
        std::env::set_current_dir("/")
            .with_context(|| format!("Failed to set current dir to /"))?;

        return Ok(());
    }
}
