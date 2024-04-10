use std::{
    env::{self, consts::OS},
    ffi::OsString,
    fs::{self, read_dir},
    mem::{size_of, zeroed},
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
    ptr::null_mut,
    usize::MAX,
};
use winapi::um::{
    sysinfoapi::{GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX},
    winbase::{GetComputerNameW, GetUserNameW},
};
use winver::WindowsVersion;

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
        "\x1b[34m######  ###### \x1b[1;33m{}\x1b[31m@\x1b[34m{}
\x1b[0;34m######  ###### \x1b[35m  \x1b[32m{}
\x1b[34m######  ###### \x1b[35m  \x1b[32m{}
               \x1b[35m  \x1b[32m{}/{} MiB({:.2}%)
\x1b[34m######  ###### \x1b[35m  \x1b[32m{}
\x1b[34m######  ###### \x1b[35m  \x1b[32m{}
\x1b[34m######  ###### \x1b[35m  \x1b[31m███\x1b[32m███\x1b[33m███\x1b[34m███\x1b[35m███\x1b[36m███\x1b[0m",
        username,
        device,
        os,
        version,
        cpu_used.to_string().as_str(),
        cpu_total.to_string().as_str(),
        ((cpu_used as f64 / cpu_total as f64) * 100.0)
            .to_string()
            .as_str(),
        packages,
        uptime,
    )
}

fn main() {
    match OS {
        "windows" => {
            // OS Name & Version
            let ver = WindowsVersion::detect().unwrap_or(WindowsVersion::new(0, 0, 0));
            let os = String::from("Windows ") + (ver.major + 1).to_string().as_str();

            // Username & Hostname
            let mut username_size = 0;
            unsafe { GetUserNameW(null_mut(), &mut username_size) };
            let mut username_buf = Vec::with_capacity(username_size.try_into().unwrap_or(MAX));
            unsafe {
                GetUserNameW(username_buf.as_mut_ptr(), &mut username_size);
                username_buf.set_len(username_size.try_into().unwrap_or(MAX));
            }
            let username = OsString::from_wide(&username_buf)
                .into_string()
                .unwrap_or(String::from("Unknown"));

            let mut devicename_size = 0;
            unsafe { GetComputerNameW(null_mut(), &mut devicename_size) };
            let mut devicename_buf = Vec::with_capacity(devicename_size.try_into().unwrap_or(MAX));
            unsafe {
                GetComputerNameW(devicename_buf.as_mut_ptr(), &mut devicename_size);
                devicename_buf.set_len(devicename_size.try_into().unwrap_or(MAX));
            }
            let devicename = OsString::from_wide(&devicename_buf)
                .into_string()
                .unwrap_or(String::from("Unknown"));

            // Uptime
            #[allow(unused_assignments)]
            let mut uptime = String::new();
            let uptime_in_secs = unsafe { GetTickCount64() / 1000 };
            let days = uptime_in_secs / (24 * 60 * 60);
            let hours = (uptime_in_secs % (24 * 60 * 60)) / (60 * 60);
            let mins = (uptime_in_secs % (24 * 60 * 60)) % (60 * 60) / 60;
            let secs = (uptime_in_secs % (24 * 60 * 60)) % (60 * 60) % 60;
            if days > 0 {
                uptime = format!("{} days, {} hours, {} minutes", days, hours, mins);
            } else if hours > 0 {
                uptime = format!("{} hours, {} minutes", hours, mins);
            } else if mins > 0 {
                uptime = format!("{} minutes, {} seconds", mins, secs);
            } else {
                uptime = format!("{} seconds", secs);
            }

            // Memory Usage
            let total_mem: u64;
            let used_mem: u64;
            let mut mem: MEMORYSTATUSEX = unsafe { zeroed() };
            mem.dwLength = size_of::<MEMORYSTATUSEX>() as _;
            let _ = unsafe { GlobalMemoryStatusEx(&mut mem) };
            total_mem = mem.ullTotalPhys as _;
            used_mem = (mem.ullTotalPhys - mem.ullAvailPhys) as _;

            let mut packages = String::new();

            // Scoop package count
            let mut scoop_count = 0;
            let mut scoop_exists = false;
            let mut scoop_path = String::new();

            if let Ok(path) = env::var("PATH") {
                for p in path.split(";") {
                    if fs::metadata(format!("{}/{}", p, "scoop")).is_ok() {
                        scoop_exists = true;
                        scoop_path = format!("{}/{}", p, "scoop");
                    }
                }
            }

            if scoop_exists {
                let mut scoop_dir = PathBuf::from(scoop_path)
                    .parent()
                    .unwrap_or(Path::new(""))
                    .to_path_buf()
                    .parent()
                    .unwrap_or(Path::new(""))
                    .to_path_buf();

                scoop_dir = scoop_dir.join("apps");
                let files = read_dir(scoop_dir.as_os_str().to_str().unwrap_or("scoop"));

                match files {
                    Ok(apps) => {
                        scoop_count = apps.count();
                    }
                    Err(_) => {}
                }
            }

            if scoop_count > 0 {
                packages += (scoop_count.to_string() + " (scoop)").as_str();
            }

            // Chocolatey package count
            let mut choco_count = 0;
            let choco_path = PathBuf::new()
                .join(env::var("PROGRAM_DATA").unwrap_or(String::from("C:\\ProgramData")))
                .join("chocolatey")
                .join("lib");

            let files = read_dir(
                choco_path
                    .to_str()
                    .unwrap_or("C:\\ProgramData\\chocolatey\\lib"),
            );

            match files {
                Ok(apps) => {
                    choco_count = apps.count();
                }
                Err(_) => {}
            }

            if choco_count > 0 {
                if scoop_count > 0 {
                    packages += ", ";
                }
                packages += (choco_count.to_string() + " (choco)").as_str();
            }

            println!(
                "{}",
                windows_ascii(
                    username.as_str(),
                    devicename.as_str(),
                    os.as_str(),
                    ver.to_string().as_str(),
                    used_mem / 1000000,
                    total_mem / 1000000,
                    packages.as_str(),
                    uptime.as_str()
                )
            );
        }
        _ => {
            println!("This runs on anything but *nix operating systems");
        }
    }
}
