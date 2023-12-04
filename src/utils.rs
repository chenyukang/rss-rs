use colored::Colorize;
use nix::sys::signal::kill;
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn check_process(pid_file: &PathBuf) -> Result<i32, ExitCode> {
    let pid_str = fs::read_to_string(pid_file).map_err(|_| ExitCode::FAILURE)?;
    let pid = pid_str
        .trim()
        .parse::<i32>()
        .map_err(|_| ExitCode::FAILURE)?;

    // Check if the process is running
    match kill(Pid::from_raw(pid), None) {
        Ok(_) => Ok(pid),
        Err(_) => Err(ExitCode::FAILURE),
    }
}

pub fn kill_process(pid_file: &PathBuf, name: &str) -> Result<(), ExitCode> {
    if check_process(pid_file).is_err() {
        eprintln!("{} is not running", name);
        return Ok(());
    }
    let pid_str = fs::read_to_string(pid_file).map_err(|_| ExitCode::FAILURE)?;
    let pid = pid_str
        .trim()
        .parse::<i32>()
        .map_err(|_| ExitCode::FAILURE)?;
    eprintln!("kill {} process {} ...", name, pid.to_string().red());
    // Send a SIGTERM signal to the process
    let _ = kill(Pid::from_raw(pid), Some(Signal::SIGTERM)).map_err(|_| ExitCode::FAILURE);
    // sleep 3 seconds and check if the process is still running
    std::thread::sleep(std::time::Duration::from_secs(3));
    match check_process(pid_file) {
        Ok(_) => kill(Pid::from_raw(pid), Some(Signal::SIGKILL)).map_err(|_| ExitCode::FAILURE),
        _ => Ok(()),
    }
}
