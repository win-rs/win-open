# win-open

[![crates.io](https://img.shields.io/crates/v/win-open.svg)](https://crates.io/crates/win-open)

The `win-open` crate allows you to open files or URLs using the default program configured on your Windows system. It supports three shells: **Command Prompt**, **PowerShell**, and **NuShell**.

## Features

- Open files or URLs with the default application on your system.
- Supports opening with specific applications by specifying the application name.
- Simple error handling in case of failure.

## Installation

Add the `win-open` crate to your `Cargo.toml`:
```toml
[dependencies]
win-open = "0.1.0"
```

## Library Usage

To open a URL or file with the default application, use the `that` function:

```Rust
win_open::that("https://rust-lang.org");
```

To open something with a specific application (for example, opening a URL with Firefox):

```Rust
win_open::with("https://rust-lang.org", "firefox");
```

## Binary Usage

In addition to the library functionality, this crate also provides a command-line tool. You can use this
tool to open files or URLs directly from the terminal.

To use the binary, run the following command:

```shell
cargo run 'file_or_url_to_open'
```

This will open the provided file or URL using the system's default program or the specified application.

## Error Handling

The crate offers basic error handling to notify you if the opening operation fails.
For more information, refer to the [API documentation](https://docs.rs/win-open).

## Credits

The implementation of this crate is based on the functionality from [Cargo](https://github.com/rust-lang/cargo), but has been improved to offer additional error handling and support for different shells on Windows.

The core idea was inspired by [open](https://github.com/Byron/open-rs), which provides a simple interface for opening files and URLs in a platform-specific manner.
