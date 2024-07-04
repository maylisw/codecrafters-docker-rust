use anyhow::{Context, Result};
use libc;
use std::{
    io::Read,
    process::{Command, Stdio},
};

pub fn run(command: &str, args: &[String]) -> Result<i32> {
    unsafe { libc::unshare(libc::CLONE_NEWPID) };

    let mut process = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to run '{}' with arguments {:?}", command, args))?;

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

    return Ok(process
        .wait()
        .with_context(|| format!("Failed to wait for process"))?
        .code()
        .unwrap_or_default());
}
