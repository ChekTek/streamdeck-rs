use anyhow::Result;
use colored::Colorize;

pub fn run(disable: bool) -> Result<()> {
    set_developer_mode(disable)?;
    let state = if disable { "disabled" } else { "enabled" };
    println!("Developer mode {}", state.green());
    Ok(())
}

pub fn set_developer_mode(disable: bool) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use anyhow::{Context, bail};

        let value = if disable { "NO" } else { "YES" };
        let status = std::process::Command::new("defaults")
            .args([
                "write",
                "com.elgato.StreamDeck",
                "developer_mode",
                "-bool",
                value,
            ])
            .status()
            .context("failed to run defaults")?;
        if !status.success() {
            bail!("failed to set developer mode");
        }
        Ok(())
    }
    #[cfg(windows)]
    {
        set_developer_mode_windows(disable)
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        let _ = disable;
        anyhow::bail!("developer mode is only supported on macOS and Windows")
    }
}

#[cfg(windows)]
fn set_developer_mode_windows(disable: bool) -> Result<()> {
    use anyhow::Context;
    use winreg::RegKey;
    use winreg::enums::*;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey("Software\\Elgato Systems GmbH\\StreamDeck")
        .context("failed to open Stream Deck registry key")?;
    let value: u32 = if disable { 0 } else { 1 };
    key.set_value("developer_mode", &value)
        .context("failed to set developer_mode")?;
    Ok(())
}
