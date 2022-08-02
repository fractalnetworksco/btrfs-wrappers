use anyhow::{anyhow, Error, Result};
use lazy_static::lazy_static;
use log::*;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};

/// Create a new, empty BTRFS subvolume at `path`.
///
/// This assumes that the parent directory of `path` is a BTRFS volume.
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

/// Take a snapshot of the BTRFS subvolume `path` at the new folder `snapshot`. Optionally,
/// mark the snapshot as read-only.
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

/// Delete a BTRFS subvolume.
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

/// Parsed output of the `btrfs subvolume show` command.
pub struct BtrfsSubvolumeShow {
    /// Current generation number of this subvolume.
    pub generation: u64,
}

/// Runs `btrfs subvolume show` and parses the output.
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

/// Start a BTRFS send process, returning a handle to it's standard output.
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

/// Start a BTRFS receive process, returning a handle to it's standard output.
pub async fn btrfs_receive(path: &Path) -> std::io::Result<Child> {
    let mut command = Command::new("btrfs");
    command.arg("receive");
    command.arg(path);
    debug!("Receiving BTRFS snapshot: {:?}", command);

    command.stdin(Stdio::piped());
    command.spawn()
}

/// Format a block device as a BTRFS filesystem.
///
/// This will overwrite the block device at `path`.
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

/// Unmount path.
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

/// Options for mounting a BTRFS filesystem.
pub struct MountOptions {
    /// Path to block device holding the filesystem.
    pub block_device: PathBuf,
    /// Folder at which the filesystem should be mounted.
    pub mount_target: PathBuf,
}

/// Mount a BTRFS filesystem at a specified path.
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
