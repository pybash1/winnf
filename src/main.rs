use colored::Colorize;
use std::{
    env::consts::OS,
    process::{Command, Stdio},
};
use sysinfo::System;
use uptime_lib::get;
use whoami::{devicename, distro, platform, username};

pub fn windows_ascii(
    username: &str,
    device: &str,
    os: &str,
    version: &str,
    cpu_used: u64,
    cpu_total: u64,
    packages: &str,
    uptime: &str,
) -> String {
    format!(
        "{} {}{}{}
{} {}  {}
{} {}  {}
               {}  {}{}{} {}{:.2}{}
{} {}  {}
{} {}  {}
{} {}  {}{}{}{}{}{}",
        "######  ######".blue(),
        username.yellow().bold(),
        "@".red().bold(),
        device.blue().bold(),
        "######  ######".blue(),
        "".magenta(),
        os.green(),
        "######  ######".blue(),
        "".magenta(),
        version.green(),
        "".magenta(),
        cpu_used.to_string().as_str().green(),
        "/".green(),
        cpu_total.to_string().as_str().green(),
        "MiB (".green(),
        ((cpu_used as f64 / cpu_total as f64) * 100.0)
            .to_string()
            .as_str()
            .green(),
        "%)".green(),
        "######  ######".blue(),
        "".magenta(),
        packages.green(),
        "######  ######".blue(),
        "".magenta(),
        uptime.green(),
        "######  ######".blue(),
        "".magenta(),
        "███".red(),
        "███".green(),
        "███".yellow(),
        "███".blue(),
        "███".magenta(),
        "███".cyan(),
    )
}

fn main() {
    let mut sys = System::new();
    sys.refresh_memory();

    let uptime = get();
    let mut uptime_val = String::new();

    match uptime {
        Ok(_) => {
            let uptime_in_secs = uptime.unwrap().as_secs();
            let days = uptime_in_secs / (24 * 60 * 60);
            let hours = (uptime_in_secs % (24 * 60 * 60)) / (60 * 60);
            let mins = (uptime_in_secs % (24 * 60 * 60)) % (60 * 60) / 60;
            let secs = (uptime_in_secs % (24 * 60 * 60)) % (60 * 60) % 60;
            if days > 0 {
                uptime_val = format!("{} days, {} hours, {} minutes", days, hours, mins);
            } else if hours > 0 {
                uptime_val = format!("{} hours, {} minutes", hours, mins);
            } else if mins > 0 {
                uptime_val = format!("{} minutes, {} seconds", mins, secs);
            } else {
                uptime_val = format!("{} seconds", secs);
            }
        }
        Err(_) => uptime_val = String::from("Unavailable"),
    }

    match OS {
        "windows" => {
            let distro = distro();
            let ver = distro.split_whitespace().collect::<Vec<&str>>()[1];
            let mut os = platform().to_string();

            os += " ";
            os += (ver.split(".").collect::<Vec<&str>>()[0]
                .parse::<i8>()
                .unwrap()
                + 1)
            .to_string()
            .as_str();

            let mut packages = String::new();

            let mut scoop_count = 0;
            let scoop_cmd = Command::new("cmd")
                .arg("/C")
                .arg("scoop list")
                .stdout(Stdio::piped())
                .output();

            match scoop_cmd {
                Ok(_) => {
                    let unwrapped = scoop_cmd.unwrap();
                    let stderr = String::from_utf8_lossy(&unwrapped.stderr);
                    let stdout = String::from_utf8_lossy(&unwrapped.stdout);
                    if stderr.contains("' is not recognized as an internal or external command,") {
                        scoop_count = 0;
                    } else {
                        scoop_count = stdout.lines().count() - 5;
                    }
                }
                Err(_) => {
                    scoop_count = 0;
                }
            }

            if scoop_count > 0 {
                packages += (scoop_count.to_string() + " (scoop)").as_str();
            }

            let mut choco_count = 0;
            let choco_cmd = Command::new("cmd")
                .arg("/C")
                .arg("choco list -l")
                .stdout(Stdio::piped())
                .output();

            match choco_cmd {
                Ok(_) => {
                    let unwrapped = choco_cmd.unwrap();
                    let stderr = String::from_utf8_lossy(&unwrapped.stderr);
                    let stdout = String::from_utf8_lossy(&unwrapped.stdout);
                    if stderr.contains("' is not recognized as an internal or external command,") {
                        choco_count = 0;
                    } else {
                        choco_count = stdout.split(".\n").collect::<Vec<&str>>()[0]
                            .lines()
                            .count()
                            - 2;
                    }
                }
                Err(_) => {
                    choco_count = 0;
                }
            }

            if choco_count > 0 {
                packages += ", ";
                packages += (choco_count.to_string() + " (choco)").as_str();
            }

            println!(
                "{}",
                windows_ascii(
                    username().as_str(),
                    devicename().as_str(),
                    os.as_str(),
                    ver,
                    sys.used_memory() / 1000000,
                    sys.total_memory() / 1000000,
                    packages.as_str(),
                    uptime_val.as_str()
                )
            );
        }
        _ => {
            println!("This runs on anything but *nix operating systems");
        }
    }
}
