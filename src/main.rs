use anyhow::Result;
use std::process;

mod docker_client;
mod filesystem;
mod runtime;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let full_image = &args[2];
    let command = &args[3];
    let command_args = &args[4..];

    let image_fs = filesystem::IsolatedFileSystem::setup(command)?;
    let client = docker_client::DockerClient::new();
    client.download_image(full_image, &image_fs.root_dir)?;
    image_fs.chroot()?;
    process::exit(runtime::run(command, command_args)?);
}
