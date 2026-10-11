#[cfg(windows)]
mod platform {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    const NO_WINDOW: u32 = 0x0800_0000;

    fn reg(args: &[&str]) -> bool {
        Command::new("reg")
            .args(args)
            .creation_flags(NO_WINDOW)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn enabled() -> bool {
        reg(&["query", KEY, "/v", "Dextra"])
    }

    pub fn set(on: bool) {
        if on {
            if let Ok(exe) = std::env::current_exe() {
                let command = format!("\"{}\" --background", exe.display());
                reg(&["add", KEY, "/v", "Dextra", "/t", "REG_SZ", "/d", &command, "/f"]);
            }
        } else {
            reg(&["delete", KEY, "/v", "Dextra", "/f"]);
        }
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn enabled() -> bool {
        false
    }

    pub fn set(_: bool) {}
}

pub use platform::{enabled, set};
