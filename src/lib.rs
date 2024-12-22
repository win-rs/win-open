//! Use this library to open a path or URL using the program configured on the system in a non-blocking fashion.
//!
//! # Usage
//!
//! Open the given URL in the default web browser, without blocking.
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! win_open::that("http://rust-lang.org")?;
//! # Ok(())
//! # }
//! ```
//!
//! Alternatively, specify the program to be used to open the path or URL.
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! win_open::with("http://rust-lang.org", "firefox")?;
//! # Ok(())
//! # }
//! ```
//!
//! Or obtain the commands to launch a file or path without running them.
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let cmds = win_open::commands("http://rust-lang.org")[0].status()?;
//! # Ok(())
//! # }
//! ```
//!
//! It's also possible to obtain a command that can be used to open a path in an application.
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let status = win_open::with_command("http://rust-lang.org", "firefox").status()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Notes
//!
//! ## Nonblocking operation
//!
//! The functions provided are nonblocking as it will return even though the
//! launched child process is still running. Note that depending on the operating
//! system, spawning launch helpers, which this library does under the hood,
//! might still take 100's of milliseconds.
//!
//! **Beware that on some platforms and circumstances, the launcher may block**.
//! In this case, please use the [`commands()`] or [`with_command()`] accordingly
//! to `spawn()` it without blocking.
//!
//! ## Error handling
//!
//! As an operating system program is used, the open operation can fail.
//! Therefore, you are advised to check the result and behave
//! accordingly, e.g. by letting the user know that the open operation failed.
//!
//! ```no_run
//! let path = "http://rust-lang.org";
//!
//! match win_open::that(path) {
//!     Ok(()) => println!("Opened '{}' successfully.", path),
//!     Err(err) => eprintln!("An error occurred when opening '{}': {}", path, err),
//! }
//! ```

#![allow(clippy::upper_case_acronyms, unused_assignments, dead_code)]
#[cfg(not(target_os = "windows"))]
compile_error!("open is not supported on this platform");

use std::{
    ffi::{OsStr, OsString},
    os::windows::process::CommandExt as WinCommandExt,
    process::{Command, Stdio},
    sync::OnceLock,
};

pub use error::Error;
use error::ErrorKind;
pub use error::Result;
pub use shell::WindowsShell;

mod error;
mod shell;

const CREATE_NO_WINDOW: u32 = 0x08000000;
static DETECTED_SHELL: OnceLock<WindowsShell> = OnceLock::new();

/// Open path with the default application without blocking.
///
/// # Examples
///
/// ```no_run
/// let path = "http://rust-lang.org";
///
/// match win_open::that(path) {
///     Ok(()) => println!("Opened '{}' successfully.", path),
///     Err(err) => panic!("An error occurred when opening '{}': {}", path, err),
/// }
/// ```
///
/// # Errors
///
/// A [`Error`] is returned on failure. Because different operating systems
/// handle errors differently it is recommend to not match on a certain error.
///
/// # Beware
///
/// Sometimes, depending on the platform and system configuration, launchers *can* block.
/// If you want to be sure they don't, use [`that_in_background()`] or [`that_detached`] instead.
pub fn that(path: impl AsRef<OsStr>) -> Result<()> {
    let mut last_err = None;
    for mut cmd in commands(path) {
        match cmd.status_without_output() {
            Ok(status) => {
                return Ok(status).into_result(cmd);
            }
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.map_or_else(
        || Error::new(ErrorKind::NO_LAUNCHER, ""),
        |err| Error::new(ErrorKind::IO, err.to_string().as_str()),
    ))
}

/// Open path with the given application.
///
/// This function may block if the application or launcher doesn't detach itself.
/// In that case, consider using [`with_in_background()`] or [`with_command()].
///
/// # Examples
///
/// ```no_run
/// let path = "http://rust-lang.org";
/// let app = "firefox";
///
/// match win_open::with(path, app) {
///     Ok(()) => println!("Opened '{}' successfully.", path),
///     Err(err) => panic!("An error occurred when opening '{}': {}", path, err),
/// }
/// ```
///
/// # Errors
///
/// A [`Error`] is returned on failure. Because different operating systems
/// handle errors differently it is recommend to not match on a certain error.
pub fn with(path: impl AsRef<OsStr>, app: impl Into<String>) -> Result<()> {
    let mut cmd = with_command(path, app);
    cmd.status_without_output().into_result(cmd)
}

/// Get multiple commands that open `path` with the default application.
///
/// Each command represents a launcher to try.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = "http://rust-lang.org";
/// assert!(win_open::commands(path)[0].status()?.success());
/// # Ok(())
/// # }
/// ```
pub fn commands<T: AsRef<OsStr>>(path: T) -> Vec<Command> {
    let shell = detect_shell().as_str();
    let mut cmd = Command::new(shell);
    match shell {
        "pwsh" => cmd
            .arg("-NoProfile")
            .arg("-Command")
            .arg("Start-Process")
            .arg(wrap_in_quotes(path.as_ref()))
            .creation_flags(CREATE_NO_WINDOW),
        "nu" => cmd
            .arg("-c")
            .arg(format!("open {}", wrap_in_quotes_string(path.as_ref())))
            .creation_flags(CREATE_NO_WINDOW),
        "cmd" => cmd
            .arg("/c")
            .arg("start")
            .raw_arg("\"\"")
            .raw_arg(wrap_in_quotes(path))
            .creation_flags(CREATE_NO_WINDOW),
        _ => panic!("No supported shell detected."),
    };
    vec![cmd]
}

/// Get a command that uses `app` to open `path`.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = "http://rust-lang.org";
/// assert!(win_open::with_command(path, "app").status()?.success());
/// # Ok(())
/// # }
/// ```
pub fn with_command<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> Command {
    let shell = detect_shell().as_str();
    let mut cmd = Command::new(shell);

    match shell {
        "pwsh" => cmd
            .arg("-NoProfile")
            .arg("-Command")
            .arg("Start-Process")
            .arg(wrap_in_quotes(path.as_ref()))
            .arg(wrap_in_quotes(app.into()))
            .creation_flags(CREATE_NO_WINDOW),
        "nu" => cmd
            .arg("-c")
            .arg(format!(
                "open {} {}",
                wrap_in_quotes_string(path.as_ref()),
                wrap_in_quotes_string(app.into())
            ))
            .creation_flags(CREATE_NO_WINDOW),
        "cmd" => cmd
            .arg("/c")
            .arg("start")
            .raw_arg("\"\"")
            .raw_arg(wrap_in_quotes(path))
            .raw_arg(wrap_in_quotes(app.into()))
            .creation_flags(CREATE_NO_WINDOW),
        _ => panic!("No supported shell detected."),
    };

    cmd
}

/// Open path with the default application in a new thread to assure it's non-blocking.
///
/// See documentation of [`that()`] for more details.
pub fn that_in_background(path: impl AsRef<OsStr>) -> std::thread::JoinHandle<Result<()>> {
    let path = path.as_ref().to_os_string();
    std::thread::spawn(|| that(path))
}

/// Open path with the given application in a new thread, which is useful if
/// the program ends up to be blocking. Otherwise, prefer [`with()`] for
/// straightforward error handling.
///
/// See documentation of [`with()`] for more details.
pub fn with_in_background<T: AsRef<OsStr>>(
    path: T,
    app: impl Into<String>,
) -> std::thread::JoinHandle<Result<()>> {
    let path = path.as_ref().to_os_string();
    let app = app.into();
    std::thread::spawn(|| with(path, app))
}

fn detect_shell() -> WindowsShell {
    *DETECTED_SHELL.get_or_init(|| match get_shell() {
        Ok(shell) => shell,
        Err(err) => {
            panic!("Failed to detect a supported shell: {}", err);
        }
    })
}

fn get_shell() -> Result<WindowsShell> {
    if Command::new("pwsh")
        .arg("-Command")
        .arg("$PSVersionTable.PSVersion")
        .status_without_output()
        .map_or(false, |status| status.success())
    {
        return "pwsh".try_into();
    }

    if Command::new("nu")
        .arg("-c")
        .arg("version")
        .status_without_output()
        .map_or(false, |status| status.success())
    {
        return "nu".try_into();
    }

    "cmd".try_into()
}

fn wrap_in_quotes<T: AsRef<OsStr>>(path: T) -> OsString {
    let mut result = OsString::from("\"");
    result.push(path);
    result.push("\"");

    result
}

fn wrap_in_quotes_string<T: AsRef<OsStr>>(path: T) -> String {
    let path = path.as_ref().to_string_lossy();
    format!("\"{}\"", path)
}

/// Open path with the default application using a detached process. which is useful if
/// the program ends up to be blocking or want to out-live your app
///
/// See documentation of [`that()`] for more details.
pub fn that_detached(path: impl AsRef<OsStr>) -> Result<()> {
    #[cfg(not(feature = "shellexecute"))]
    {
        let mut last_err = None;
        for mut cmd in commands(path) {
            match cmd.spawn_detached() {
                Ok(_) => {
                    return Ok(());
                }
                Err(err) => last_err = Some(err),
            }
        }
        Err(last_err.map_or_else(
            || Error::new(ErrorKind::NO_LAUNCHER, ""),
            |err| Error::new(ErrorKind::IO, err.to_string().as_str()),
        ))
    }

    #[cfg(feature = "shellexecute")]
    {
        that_detached_execute(path)
    }
}

/// Open path with the given application using a detached process, which is useful if
/// the program ends up to be blocking or want to out-live your app. Otherwise, prefer [`with()`] for
/// straightforward error handling.
///
/// See documentation of [`with()`] for more details.
pub fn with_detached<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> Result<()> {
    #[cfg(not(feature = "shellexecute"))]
    {
        let mut last_err = None;
        let mut cmd = with_command(path, app);

        // Try spawning the detached process
        match cmd.spawn_detached() {
            Ok(_) => {
                return Ok(()); // Successfully spawned the detached process
            }
            Err(err) => {
                last_err = Some(err); // Store the error if spawning fails
            }
        }

        Err(last_err.map_or_else(
            || Error::new(ErrorKind::NO_LAUNCHER, ""),
            |err| Error::new(ErrorKind::IO, err.to_string().as_str()),
        ))
    }

    #[cfg(feature = "shellexecute")]
    {
        with_detached_execute(path, app)
    }
}

trait IntoResult<T> {
    fn into_result(self, cmd: Command) -> T;
}

impl IntoResult<Result<()>> for std::io::Result<std::process::ExitStatus> {
    fn into_result(self, cmd: Command) -> Result<()> {
        match self {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(Error::new(
                ErrorKind::COMMAND_FAILED,
                format!("{cmd:?} ({})", status).as_str(),
            )),
            Err(err) => Err(err.into()),
        }
    }
}

trait CommandExt {
    fn status_without_output(&mut self) -> std::io::Result<std::process::ExitStatus>;
    fn spawn_detached(&mut self) -> std::io::Result<()>;
}

impl CommandExt for Command {
    fn status_without_output(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    }

    fn spawn_detached(&mut self) -> std::io::Result<()> {
        // This is pretty much lifted from the implementation in Alacritty:
        // https://github.com/alacritty/alacritty/blob/b9c886872d1202fc9302f68a0bedbb17daa35335/alacritty/src/daemon.rs

        self.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        self.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
        self.spawn().map(|_| ())
    }
}

#[cfg(feature = "shellexecute")]
fn that_detached_execute<T: AsRef<OsStr>>(path: T) -> Result<()> {
    let path = path.as_ref();
    let is_dir = std::fs::metadata(path).map(|f| f.is_dir()).unwrap_or(false);

    let path = wide(path);

    if is_dir {
        unsafe { ffi::CoInitialize(std::ptr::null()) };
        let folder = unsafe { ffi::ILCreateFromPathW(path.as_ptr()) };
        unsafe { SHOpenFolderAndSelectItems(folder, Some(&[folder]), 0)? };
        return Ok(());
    };

    let mut info = ffi::SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<ffi::SHELLEXECUTEINFOW>() as _,
        nShow: ffi::SW_SHOWNORMAL,
        lpVerb: std::ptr::null(),
        lpClass: std::ptr::null(),
        lpFile: path.as_ptr(),
        ..unsafe { std::mem::zeroed() }
    };

    unsafe { ShellExecuteExW(&mut info) }
}

#[cfg(feature = "shellexecute")]
pub fn with_detached_execute<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> Result<()> {
    let app = wide(app.into());
    let path = wide(path);

    let mut info = ffi::SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<ffi::SHELLEXECUTEINFOW>() as _,
        nShow: ffi::SW_SHOWNORMAL,
        lpFile: app.as_ptr(),
        lpParameters: path.as_ptr(),
        ..unsafe { std::mem::zeroed() }
    };

    unsafe { ShellExecuteExW(&mut info) }
}

/// Encodes as wide and adds a null character.
#[cfg(feature = "shellexecute")]
#[inline]
fn wide<T: AsRef<OsStr>>(input: T) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    input
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Performs an operation on a specified file.
///
/// <https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecuteexw>
///
/// # Safety
/// This function is unsafe because it interacts with raw pointers to the `SHELLEXECUTEINFOW` structure.
/// The caller must ensure that:
/// - The pointer `info` is valid and points to a properly initialized `SHELLEXECUTEINFOW` structure.
/// - The `SHELLEXECUTEINFOW` structure should be correctly populated according to the Windows API documentation.
/// - The memory referenced by the `info` pointer must remain valid for the duration of the function call.
///
/// Failing to meet these conditions could result in undefined behavior, such as dereferencing invalid memory
/// or passing incorrect data to the Windows API, which could lead to incorrect results or crashes.
#[allow(non_snake_case)]
#[cfg(feature = "shellexecute")]
unsafe fn ShellExecuteExW(info: *mut ffi::SHELLEXECUTEINFOW) -> Result<()> {
    // ShellExecuteExW returns TRUE (i.e 1) on success
    // https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecuteexw#remarks
    if ffi::ShellExecuteExW(info) == 1 {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::IO,
            std::io::Error::last_os_error().to_string().as_str(),
        ))
    }
}

// Taken from https://microsoft.github.io/windows-docs-rs/doc/windows/
/// Opens a Windows Explorer window with specified items in a particular folder selected.
///
/// <https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shopenfolderandselectitems>
///
/// # Safety
/// This function is unsafe because it interacts with raw pointers and calls a Windows API
/// function that expects valid, non-null pointers to `ITEMIDLIST` structures. The caller
/// must ensure that:
/// - `pidlfolder` points to a valid `ITEMIDLIST` structure.
/// - `apidl` points to a valid slice of pointers to `ITEMIDLIST` structures (if provided).
/// - The pointers passed must remain valid for the duration of the function call, as they are used directly by the Windows API.
///
/// Failing to adhere to these safety guarantees could result in undefined behavior, such as dereferencing invalid memory.
#[allow(non_snake_case)]
#[cfg(feature = "shellexecute")]
unsafe fn SHOpenFolderAndSelectItems(
    pidlfolder: *const ffi::ITEMIDLIST,
    apidl: Option<&[*const ffi::ITEMIDLIST]>,
    dwflags: u32,
) -> Result<()> {
    use std::convert::TryInto;

    match ffi::SHOpenFolderAndSelectItems(
        pidlfolder,
        apidl.map_or(0, |slice| slice.len().try_into().unwrap()),
        apidl.map_or(core::ptr::null(), |slice| slice.as_ptr()),
        dwflags,
    ) {
        0 => Ok(()),
        error_code => Err(Error::new(
            ErrorKind::IO,
            std::io::Error::from_raw_os_error(error_code)
                .to_string()
                .as_str(),
        )),
    }
}

#[cfg(feature = "shellexecute")]
#[allow(non_snake_case)]
mod ffi {
    /// Activates and displays a window.
    /// If the window is minimized, maximized, or arranged, the system restores it to its original size and position.
    /// An application should specify this flag when displaying the window for the first time.
    ///
    /// <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow>
    pub const SW_SHOWNORMAL: i32 = 1;

    // Taken from https://docs.rs/windows-sys/latest/windows_sys/
    #[cfg_attr(not(target_arch = "x86"), repr(C))]
    #[cfg_attr(target_arch = "x86", repr(C, packed(1)))]
    pub struct SHELLEXECUTEINFOW {
        pub cbSize: u32,
        pub fMask: u32,
        pub hwnd: isize,
        pub lpVerb: *const u16,
        pub lpFile: *const u16,
        pub lpParameters: *const u16,
        pub lpDirectory: *const u16,
        pub nShow: i32,
        pub hInstApp: isize,
        pub lpIDList: *mut core::ffi::c_void,
        pub lpClass: *const u16,
        pub hkeyClass: isize,
        pub dwHotKey: u32,
        pub Anonymous: SHELLEXECUTEINFOW_0,
        pub hProcess: isize,
    }

    // Taken from https://docs.rs/windows-sys/latest/windows_sys/
    #[cfg_attr(not(target_arch = "x86"), repr(C))]
    #[cfg_attr(target_arch = "x86", repr(C, packed(1)))]
    pub union SHELLEXECUTEINFOW_0 {
        pub hIcon: isize,
        pub hMonitor: isize,
    }

    // Taken from https://microsoft.github.io/windows-docs-rs/doc/windows/
    #[repr(C, packed(1))]
    pub struct SHITEMID {
        pub cb: u16,
        pub abID: [u8; 1],
    }

    // Taken from https://microsoft.github.io/windows-docs-rs/doc/windows/
    #[repr(C, packed(1))]
    pub struct ITEMIDLIST {
        pub mkid: SHITEMID,
    }

    #[link(name = "shell32")]
    extern "system" {
        pub fn ShellExecuteExW(info: *mut SHELLEXECUTEINFOW) -> isize;
        pub fn ILCreateFromPathW(pszpath: *const u16) -> *mut ITEMIDLIST;
        pub fn SHOpenFolderAndSelectItems(
            pidlfolder: *const ITEMIDLIST,
            cidl: u32,
            apidl: *const *const ITEMIDLIST,
            dwflags: u32,
        ) -> i32;
    }

    #[link(name = "ole32")]
    extern "system" {
        pub fn CoInitialize(pvreserved: *const core::ffi::c_void) -> i32;
    }
}
