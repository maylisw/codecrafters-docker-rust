use anyhow::{Context, Result};
use std::{
    io::Read,
    process::{Command, Stdio},
};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    // spawn new process
    let process = Command::new(command)
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
        Some(mut output) => {
            let mut std_out = String::new();
            output
                .read_to_string(&mut std_out)
                .with_context(|| format!("Failed to read output '{:#?}' to string", output))?;
            print!("{}", std_out);
        }
        None => todo!(),
    };

    match process.stderr {
        Some(mut errput) => {
            let mut std_err = String::new();
            errput
                .read_to_string(&mut std_err)
                .with_context(|| format!("Failed to read errput '{:#?}' to string", errput))?;
            eprint!("{}", std_err);
        }
        None => todo!(),
    };

    Ok(())
}
