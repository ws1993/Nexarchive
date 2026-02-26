use anyhow::Result;

#[cfg(windows)]
use winreg::{
    enums::{HKEY_CURRENT_USER, KEY_WRITE},
    RegKey,
};

pub struct SystemService;

impl SystemService {
    pub fn apply_autostart(enabled: bool) -> Result<()> {
        #[cfg(windows)]
        {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let (run_key, _) = hkcu.create_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                KEY_WRITE,
            )?;

            let value_name = "NexArchive";
            if enabled {
                let exe = std::env::current_exe()?;
                run_key.set_value(value_name, &exe.to_string_lossy().to_string())?;
            } else {
                let _ = run_key.delete_value(value_name);
            }
        }

        Ok(())
    }
}
