/// Enum representing various types of errors that may occur in a shell operation.
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)] // To allow the use of all-uppercase error kind variants
pub enum ErrorKind {
    /// Error indicating that a shell type was not found or recognized.
    SHELL_NOT_FOUND,

    /// Error indicating that a command execution failed.
    COMMAND_FAILED,

    /// Error indicating that no valid launcher was found.
    NO_LAUNCHER,

    /// Error indicating an I/O operation failure.
    IO,
}

/// A struct representing an error in shell operations.
/// It includes the type of the error (`ErrorKind`) and an optional message.
pub struct Error {
    kind: ErrorKind, // The type of the error (e.g., I/O, command failure)
    message: String, // An optional message describing the error
}

impl PartialEq for Error {
    /// Compares two `Error` instances for equality based on their kind.
    ///
    /// # Parameters
    /// - `self`: The first `Error` instance.
    /// - `other`: The second `Error` instance.
    ///
    /// # Returns
    /// `true` if both errors have the same kind, `false` otherwise.
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for Error {} // `Error` instances can be compared for equality using the `==` operator.

impl Error {
    /// Creates a new `Error` instance with a specified kind and an optional message.
    /// If the message is empty, it will use an empty message for the error.
    ///
    /// # Parameters
    /// - `kind`: The type of error (e.g., `IO`, `SHELL_NOT_FOUND`).
    /// - `message`: A message describing the error. This can be an empty string.
    ///
    /// # Returns
    /// A new `Error` instance with the specified error kind and message.
    pub fn new<T: AsRef<str>>(kind: ErrorKind, message: T) -> Self {
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

    /// Creates a new `Error` instance with the specified error kind and an empty message.
    ///
    /// # Parameters
    /// - `kind`: The type of error (e.g., `IO`, `SHELL_NOT_FOUND`).
    ///
    /// # Returns
    /// A new `Error` instance with an empty message for the specified error kind.
    fn empty_message(kind: ErrorKind) -> Self {
        Self {
            kind,
            message: "".to_string(),
        }
    }

    /// Retrieves the kind of the error.
    ///
    /// # Returns
    /// A reference to the `ErrorKind` variant that represents the type of error.
    pub const fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Retrieves the error message, if provided.
    ///
    /// # Returns
    /// The error message as a string slice. If no message is provided, an empty string is returned.
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl core::fmt::Display for ErrorKind {
    /// Formats the `ErrorKind` enum into a human-readable string for display purposes.
    ///
    /// This is used when the error kind is displayed directly (e.g., in an error message).
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorKind::SHELL_NOT_FOUND => {
                write!(f, "Unrecognized shell type")
            }
            ErrorKind::COMMAND_FAILED => {
                write!(f, "Command failed")
            }
            ErrorKind::NO_LAUNCHER => {
                write!(f, "No launcher worked")
            }
            ErrorKind::IO => {
                write!(f, "IO Error")
            }
        }
    }
}

impl core::fmt::Debug for Error {
    /// Formats the `Error` struct for debugging.
    ///
    /// This method provides a detailed output of the error kind and the associated message,
    /// which is useful for debugging purposes.
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug = fmt.debug_struct("Error");
        debug
            .field("kind", &self.kind())
            .field("message", &self.message())
            .finish()
    }
}

impl core::fmt::Display for Error {
    /// Formats the `Error` struct for user-facing display.
    ///
    /// If a message is provided, it includes the message along with the error kind.
    /// If no message is provided, only the error kind is displayed.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.message().is_empty() {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "{} ({})", self.kind, self.message)
        }
    }
}

impl From<std::io::Error> for Error {
    /// Converts a `std::io::Error` into an `Error` with the `IO` error kind and the error message.
    ///
    /// This allows for easy conversion from I/O errors (e.g., file or network errors) to our custom `Error` type.
    ///
    /// # Parameters
    /// - `err`: A `std::io::Error` instance that we want to convert.
    ///
    /// # Returns
    /// A new `Error` instance with the `IO` error kind and the I/O error message.
    fn from(err: std::io::Error) -> Self {
        Self::new(ErrorKind::IO, err.to_string().as_str())
    }
}

impl std::error::Error for Error {} // Implements the standard error trait for compatibility with other error handling mechanisms.

/// A custom `Result` type that returns `Error` in case of failure.
///
/// This type is used for handling errors related to shell operations. It wraps the standard `Result` type but replaces the error type with our custom `Error` type.
pub type Result<T> = core::result::Result<T, Error>;
