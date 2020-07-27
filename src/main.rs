use anyhow::Result;
use log::info;
use std::{
    fs,
    path::{Path, PathBuf},
    process,
    str::FromStr,
};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
#[structopt(rename_all = "kebab-case")]
struct Cli {
    #[structopt(long, short)]
    /// Enable verbose log output
    // TODO: Make this an incrementing counter, to support -v, -vv, etc.
    verbose: bool,

    // Register subcommands
    #[structopt(subcommand)]
    cmd: Cmd,
}

// Subcommands
#[derive(StructOpt, Debug)]
enum Cmd {
    /// Create a new vault directory on a partition
    Create {
        #[structopt(short, long)]
        /// Device partition to create the vault directory on
        device: PathBuf,

        #[structopt(short, long)]
        /// Path on the filesystem where the directory should be mounted after creation
        mountpoint: PathBuf,
    },

    /// Mount an existing vault directory in the filesystem
    Mount {
        #[structopt(short, long)]
        /// Device partition to create the vault directory on
        device: PathBuf,

        #[structopt(short, long)]
        /// Path on the filesystem where the directory should be mounted after creation
        mountpoint: PathBuf,
    },

    /// Umount a vault from the filesystem and close the LUKS volume
    Umount {
        #[structopt(short, long)]
        /// Path on the filesystem where the directory should be mounted after creation
        mountpoint: PathBuf,
    },
}

fn main() -> Result<()> {
    // Parse the input args
    let args = Cli::from_args();

    // Figure out what log level to use
    let log_level = match &args.verbose {
        true => "info",
        false => "warn",
    };

    // Set up the logger to output more nicely formatted logs at the right log level based on
    // whether the verbose flag is set
    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level));
    builder.init();
    info!("logging at level {}", log_level);

    match &args.cmd {
        Cmd::Create { device, mountpoint } => {
            // Format the device
            luks_format(device)?;

            // Open the device
            let luks_dev = luks_open(device, mountpoint.file_name().unwrap().to_str().unwrap())?;

            // Create filesystem on the device
            mkfs(&luks_dev)?;

            // Ensure the mountpoint exists and mount it
            fs::create_dir_all(mountpoint)?;
            mount(&luks_dev, &mountpoint)?;
            println!("{:?} now mounted at {:?}", device, mountpoint);
        }
        Cmd::Mount { device, mountpoint } => {
            // Open the device
            let luks_dev = luks_open(device, mountpoint.file_name().unwrap().to_str().unwrap())?;

            // Ensure the mountpoint exists and mount it
            fs::create_dir_all(mountpoint)?;
            mount(&luks_dev, &mountpoint)?;
            println!("{:?} now mounted at {:?}", device, mountpoint);
        }
        Cmd::Umount { mountpoint } => {
            // Unmount the fs
            umount(mountpoint)?;

            // Close the luks vol
            let mapper_path = format!(
                "/dev/mapper/{}",
                mountpoint.file_name().unwrap().to_str().unwrap()
            );
            luks_close(&PathBuf::from_str(&mapper_path)?)?;
            println!("{:?} now unmounted", mountpoint);
        }
    }

    Ok(())
}

fn mount(device: &Path, mountpoint: &Path) -> Result<()> {
    process::Command::new("mount")
        .arg(device)
        .arg(mountpoint)
        .output()?;
    info!("mounted device {:?} at {:?}", device, mountpoint);

    Ok(())
}

fn umount(mountpoint: &Path) -> Result<()> {
    process::Command::new("umount").arg(mountpoint).output()?;
    info!("unmounted {:?}", mountpoint);

    Ok(())
}

fn luks_open(device: &Path, name: &str) -> Result<PathBuf> {
    process::Command::new("cryptsetup")
        .arg("luksOpen")
        .arg(device)
        .arg(name)
        .output()?;

    let path = PathBuf::from_str(&format!("/dev/mapper/{}", name))?;
    info!("luks: opened device {:?} at {:?}", device, &path);

    Ok(path)
}

fn luks_close(device: &Path) -> Result<()> {
    process::Command::new("cryptsetup")
        .arg("luksClose")
        .arg(device)
        .output()?;
    info!("luks: closed {:?}", device);

    Ok(())
}

fn luks_format(device: &Path) -> Result<()> {
    process::Command::new("cryptsetup")
        .arg("luksFormat")
        .arg(device)
        .output()?;
    info!("luks: formatted {:?}", device);

    Ok(())
}

fn mkfs(device: &Path) -> Result<()> {
    process::Command::new("mkfs")
        .arg("-t")
        .arg("ext4")
        .arg(device)
        .output()?;
    info!("ext4: created filesystem on {:?}", device);

    Ok(())
}
