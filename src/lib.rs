use anyhow::{anyhow, Error, Result};
use lazy_static::lazy_static;
use log::*;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};

pub async fn btrfs_subvolume_create(path: &Path) -> Result<()> {
    let mut command = Command::new("btrfs");
    command.arg("subvolume").arg("create").arg(path);
    debug!("Creating BTRFS subvolume: {:?}", command);
    let output = command.output().await.unwrap();
    if !output.status.success() {
        return Err(anyhow!("Error creating BTRFS subvolume: {output:?}"));
    }
    Ok(())
}

pub async fn btrfs_subvolume_snapshot(path: &Path, snapshot: &Path, readonly: bool) -> Result<()> {
    let mut command = Command::new("btrfs");
    command.arg("subvolume").arg("snapshot");
    if readonly {
        command.arg("-r");
    }
    command.arg(path).arg(snapshot);
    debug!("Snapshotting BTRFS volume: {:?}", command);
    let output = command.output().await.unwrap();
    if !output.status.success() {
        return Err(anyhow!("Error snapshotting subvolume: {:?}", output));
    }
    Ok(())
}

pub async fn btrfs_subvolume_delete(path: &Path) -> Result<()> {
    let mut command = Command::new("btrfs");
    command.arg("subvolume").arg("delete").arg(path);
    debug!("Deleting BTRFS volume: {:?}", command);
    let output = command.output().await.unwrap();
    if !output.status.success() {
        return Err(anyhow!("Error deleting subvolume: {:?}", output));
    }
    Ok(())
}

pub struct BtrfsSubvolumeShow {
    pub generation: u64,
}

pub async fn btrfs_subvolume_show(path: &Path) -> Result<BtrfsSubvolumeShow, Error> {
    lazy_static! {
        static ref GENERATION: Regex = Regex::new(r"Generation:\s+(\d+)").unwrap();
    }

    let mut command = Command::new("btrfs");
    command.arg("subvolume").arg("show").arg(path);
    debug!("Getting BTRFS subvolume info: {command:?}");

    let output = command.output().await?;

    if !output.status.success() {
        return Err(anyhow!("Error getting subvolume info: {:?}", output));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let generation: u64 = GENERATION
        .captures(&stdout)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .parse()?;

    Ok(BtrfsSubvolumeShow { generation })
}

pub async fn btrfs_send(path: &Path, parent: Option<&Path>) -> std::io::Result<Child> {
    let mut command = Command::new("btrfs");
    command.arg("send");

    if let Some(parent) = parent {
        command.arg("-p").arg(parent);
    }

    command.arg(path);
    debug!("Sending BTRFS snapshot: {:?}", command);

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.spawn()
}

pub async fn btrfs_receive(path: &Path) -> std::io::Result<Child> {
    let mut command = Command::new("btrfs");
    command.arg("receive");
    command.arg(path);
    debug!("Receiving BTRFS snapshot: {:?}", command);

    command.stdin(Stdio::piped());
    command.spawn()
}

pub async fn mkfs_btrfs(path: &Path) -> Result<()> {
    let mut command = Command::new("mkfs.btrfs");
    command.arg(path);
    debug!("Creating BTRFS filesystem: {:?}", command);
    let output = command.output().await?;
    if !output.status.success() {
        Err(anyhow!("Error creating BTRFS filesystem: {:?}", output))
    } else {
        Ok(())
    }
}

/// Unmount path
pub async fn umount(path: &Path) -> Result<()> {
    let mut command = Command::new("umount");
    command.arg(path);
    debug!("Unmounting filesystem: {:?}", command);
    let output = command.output().await?;
    if !output.status.success() {
        Err(anyhow!("Error unmounting filesystem: {:?}", output))
    } else {
        Ok(())
    }
}

pub struct MountOptions {
    pub block_device: PathBuf,
    pub mount_target: PathBuf,
}

pub async fn mount_btrfs(options: MountOptions) -> Result<()> {
    let mut command = Command::new("mount");
    command
        .arg("-t")
        .arg("btrfs")
        .arg(options.block_device)
        .arg(options.mount_target);
    debug!("Mounting BTRFS filesystem: {:?}", command);
    let output = command.output().await?;
    if !output.status.success() {
        Err(anyhow!("Error mounting BTRFS filesystem: {:?}", output))
    } else {
        Ok(())
    }
}
