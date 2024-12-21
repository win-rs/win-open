use std::fmt::Debug;
use std::str::FromStr;

use crate::error::WindowsShellError;

#[derive(Debug, Copy, Clone)]
pub enum WindowsShell {
    Powershell,
    Nushell,
    Cmd,
}

impl WindowsShell {
    pub fn as_str(self) -> &'static str {
        match self {
            WindowsShell::Powershell => "pwsh",
            WindowsShell::Nushell => "nu",
            WindowsShell::Cmd => "cmd",
        }
    }
}

impl TryInto<WindowsShell> for &str {
    type Error = WindowsShellError;

    fn try_into(self) -> Result<WindowsShell> {
        match self.to_ascii_uppercase().as_str() {
            "PWSH" | "POWERSHELL" => Ok(WindowsShell::Powershell),
            "NU" | "NUSHELL" => Ok(WindowsShell::Nushell),
            "CMD" | "COMMANDPROMPT" => Ok(WindowsShell::Cmd),
            _ => Err(WindowsShellError::ShellNotFound(self.to_string())),
        }
    }
}

impl FromStr for WindowsShell {
    type Err = WindowsShellError;

    fn from_str(shell: &str) -> Result<Self> {
        shell.try_into()
    }
}

pub type Result<T> = std::result::Result<T, WindowsShellError>;
