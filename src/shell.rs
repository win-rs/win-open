use crate::error::{Error, ErrorKind, Result};
use std::fmt::Debug;
use std::str::FromStr;

/// Enum representing the different types of Windows shells that can be used.
#[derive(Debug, Copy, Clone)]
pub enum WindowsShell {
    /// PowerShell (`pwsh`).
    Powershell,

    /// Nushell (`nu`).
    Nushell,

    /// Command Prompt (`cmd`).
    Cmd,
}

impl WindowsShell {
    /// Converts a `WindowsShell` variant into its corresponding shell command as a string.
    ///
    /// This method returns the string that represents the shell command for each variant.
    ///
    /// # Returns
    /// A string slice representing the shell command (e.g., "pwsh", "nu", "cmd").
    pub fn as_str(self) -> &'static str {
        match self {
            WindowsShell::Powershell => "pwsh", // PowerShell command
            WindowsShell::Nushell => "nu",      // Nushell command
            WindowsShell::Cmd => "cmd",         // Command Prompt command
        }
    }
}

impl TryInto<WindowsShell> for &str {
    type Error = Error;

    /// Attempts to convert a string slice (`&str`) into a `WindowsShell` enum.
    ///
    /// This method tries to match the input string (case insensitive) to a valid shell type.
    /// If the input string matches one of the supported shell types, it returns the corresponding `WindowsShell` variant.
    /// If not, it returns an error indicating that the shell was not recognized.
    ///
    /// # Parameters
    /// - `self`: The input string representing the shell type to be converted.
    ///
    /// # Returns
    /// - `Ok(WindowsShell::Powershell)` if the input matches "PWSH" or "POWERSHELL".
    /// - `Ok(WindowsShell::Nushell)` if the input matches "NU" or "NUSHELL".
    /// - `Ok(WindowsShell::Cmd)` if the input matches "CMD" or "COMMANDPROMPT".
    /// - `Err(Error)` if the input does not match any known shell types.
    fn try_into(self) -> Result<WindowsShell> {
        match self.to_ascii_uppercase().as_str() {
            "PWSH" | "POWERSHELL" => Ok(WindowsShell::Powershell),
            "NU" | "NUSHELL" => Ok(WindowsShell::Nushell),
            "CMD" | "COMMANDPROMPT" => Ok(WindowsShell::Cmd),
            _ => Err(Error::new(ErrorKind::SHELL_NOT_FOUND, self)), // Error if shell is not found
        }
    }
}

impl FromStr for WindowsShell {
    type Err = Error;

    /// Attempts to parse a string slice into a `WindowsShell` enum.
    ///
    /// This method uses `try_into` to convert the input string into a corresponding `WindowsShell` variant.
    /// It returns a `Result` containing either a valid `WindowsShell` or an error if the string is not recognized.
    ///
    /// # Parameters
    /// - `shell`: The string slice representing the shell type.
    ///
    /// # Returns
    /// - `Ok(WindowsShell)` if the string matches a valid shell type.
    /// - `Err(Error)` if the string does not match any known shell types.
    fn from_str(shell: &str) -> Result<Self> {
        shell.try_into() // Delegate the conversion to the `try_into` implementation
    }
}
