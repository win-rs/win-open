#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum WindowsShellErrorKind {
    SHELL_NOT_FOUND,
    COMMAND_FAILED,
    NO_LAUNCHER,
    IO,
}

pub struct WindowsShellError {
    kind: WindowsShellErrorKind,
    message: String,
}

impl PartialEq for WindowsShellError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for WindowsShellError {}

impl WindowsShellError {
    pub fn new<T: AsRef<str>>(kind: WindowsShellErrorKind, message: T) -> Self {
        let message: &str = message.as_ref();
        if message.is_empty() {
            Self::empty_message(kind)
        } else {
            Self {
                kind,
                message: message.to_string(),
            }
        }
    }

    fn empty_message(kind: WindowsShellErrorKind) -> Self {
        Self {
            kind,
            message: "".to_string(),
        }
    }

    /// Retrieves the kind of the error.
    pub const fn kind(&self) -> &WindowsShellErrorKind {
        &self.kind
    }

    /// Retrieves the optional custom message.
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl core::fmt::Display for WindowsShellErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WindowsShellErrorKind::SHELL_NOT_FOUND => {
                write!(f, "Unrecognized shell type")
            }
            WindowsShellErrorKind::COMMAND_FAILED => {
                write!(f, "Command failed")
            }
            WindowsShellErrorKind::NO_LAUNCHER => {
                write!(f, "No launcher worked")
            }
            WindowsShellErrorKind::IO => {
                write!(f, "IO Error")
            }
        }
    }
}

impl core::fmt::Debug for WindowsShellError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug = fmt.debug_struct("WindowsShellError");
        debug
            .field("kind", &self.kind())
            .field("message", &self.message())
            .finish()
    }
}

impl core::fmt::Display for WindowsShellError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.message().is_empty() {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "{} ({})", self.kind, self.message)
        }
    }
}

impl From<std::io::Error> for WindowsShellError {
    fn from(err: std::io::Error) -> Self {
        Self::new(WindowsShellErrorKind::IO, err.to_string().as_str())
    }
}

impl std::error::Error for WindowsShellError {}

pub type WindowsShellResult<T> = core::result::Result<T, WindowsShellError>;
