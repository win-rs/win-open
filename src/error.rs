use std::process::{Command, ExitStatus};

pub enum WindowsShellError {
    ShellNotFound(String),
    NoLauncher,
    CommandFailed(Command, ExitStatus),
    Io(std::io::Error),
}

impl std::fmt::Display for WindowsShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowsShellError::ShellNotFound(shell) => {
                write!(f, "Unrecognized shell type: {}", shell)
            }
            WindowsShellError::CommandFailed(command, status) => {
                write!(f, "Launcher {command:?} failed with {:?}", status)
            }
            WindowsShellError::Io(err) => {
                write!(f, "IO Error: {}", err)
            }
            WindowsShellError::NoLauncher => {
                write!(f, "No launcher worked. At least one error.")
            }
        }
    }
}

impl std::fmt::Debug for WindowsShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WindowsShellError::{}", self)
    }
}

impl std::error::Error for WindowsShellError {}

impl From<std::io::Error> for WindowsShellError {
    fn from(err: std::io::Error) -> Self {
        WindowsShellError::Io(err)
    }
}
