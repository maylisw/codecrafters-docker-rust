use anyhow::{Context, Result};
use std::{
    fs,
    io::Read,
    os::unix,
    path::PathBuf,
    process::{self, Command, Stdio},
};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    setup_fs_isolation(&command).expect("Failed to setup sandboxed filesystem");

    unsafe { libc::unshare(libc::CLONE_NEWPID) };

    // spawn new process
    let mut process = Command::new(command)
        .args(command_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| {
            format!(
                "Failed to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    match process.stdout {
        Some(ref mut output) => {
            let mut std_out = String::new();
            output
                .read_to_string(&mut std_out)
                .with_context(|| format!("Failed to read output '{:#?}' to string", output))?;
            print!("{}", std_out);
        }
        None => todo!(),
    };

    match process.stderr {
        Some(ref mut errput) => {
            let mut std_err = String::new();
            errput
                .read_to_string(&mut std_err)
                .with_context(|| format!("Failed to read errput '{:#?}' to string", errput))?;
            eprint!("{}", std_err);
        }
        None => todo!(),
    };

    process::exit(
        process
            .wait()
            .with_context(|| format!("Failed to wait for process"))?
            .code()
            .unwrap_or_default(),
    );
}

fn setup_fs_isolation(command: &String) -> Result<(), anyhow::Error> {
    // sandbox dir
    let sandbox_dir = PathBuf::from("./sandbox");
    fs::create_dir_all(&sandbox_dir)
        .with_context(|| format!("Failed to create '{:#?}' sandbox directory", sandbox_dir))?;

    // /dev/null
    let dev = "dev";
    fs::create_dir_all(sandbox_dir.join(dev))
        .with_context(|| format!("Failed to create '{:#?}' directory", sandbox_dir.join(dev)))?;
    fs::write("/dev/null", b"").with_context(|| format!("Failed to create '/dev/null' file"))?;

    // copy in command
    let command_path = sandbox_dir.join(
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
        sandbox_dir.join(PathBuf::from(command).strip_prefix("/")?),
    )
    .with_context(|| format!("Failed to copy '{}'", command))?;

    // change root to sandbox_dir
    unix::fs::chroot(&sandbox_dir)
        .with_context(|| format!("Failed to chroot '{:#?}' sandbox directory", sandbox_dir))?;
    std::env::set_current_dir("/").with_context(|| format!("Failed to set current dir to /"))?;

    return Ok(());
}
