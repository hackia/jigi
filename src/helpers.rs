use std::{
    fs::File,
    io::{Write, stdout},
    time::Instant,
};

use crossterm::style::{Color, PrintStyledContent, Stylize};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use reqwest::blocking::get;

/// Executes a given command and optionally clears the terminal while displaying a success message.
///
/// # Parameters
///
/// * `message` - A non-empty string to display as a success message upon command execution.
/// * `clear` - A boolean value indicating whether to clear the terminal before executing the command.
/// * `command` - A mutable reference to a `std::process::Command` object representing the command to be executed.
///
/// # Panics
///
/// The function will panic if:
/// - `message` is an empty string.
/// - The terminal fails to clear when `clear` is set to `true`.
/// - The command fails to execute successfully.
///
/// # Behavior
///
/// - If `clear` is `true`, the terminal is cleared using the `crossterm` library before executing the command.
/// - The success state of the command is checked using `status().success()`.
/// - If the command executes successfully, the `ok_clear` function is called with the provided `message` and `false` as arguments.
/// - If the command execution fails, the function panics with an error message.
///
/// # Dependencies
///
/// This function relies on the `crossterm` crate for terminal clearing functionality
/// (specifically, the `Clear` and `MoveTo` executables).
///
/// # Example
///
/// ```rust
/// use std::process::Command;
///
/// let mut cmd = Command::new("echo");
/// cmd.arg("Hello, World!");
/// ok_command("Command executed successfully", true, &mut cmd);
/// ```
///
/// In this example:
/// - The terminal will be cleared.
/// - The message "Command executed successfully" will be displayed if the command runs successfully.
/// - If the command fails to execute, the program will panic.
pub fn ok_command(message: &str, clear: bool, command: &mut std::process::Command) {
    if message.is_empty() {
        panic!("Message is empty");
    }
    if clear {
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).expect("Failed to clear terminal");
    }
    if command
        .current_dir(".")
        .status()
        .expect("Failed to execute command")
        .success()
    {
        ok_clear(message, false);
    } else {
        panic!("Command failed to execute successfully");
    }
}

/// A utility function to print a styled success message to the terminal, with an optional
/// option to clear the terminal screen before displaying the message.
///
/// # Parameters
/// - `message`:
///   A string slice that contains the message to print. The function will panic
///   if the message is empty.
/// - `clear`:
///   A boolean value to indicate whether to clear the terminal screen before
///   printing the message. If set to `true`, the screen is cleared.
///
/// # Behavior
/// 1. If the `message` parameter is an empty string, the function panics with a
///    message `"Message is empty"`.
/// 2. If `clear` is `true`, the terminal screen is cleared, and the cursor moves
///    to the top-left corner.
/// 3. The provided `message` is styled with the following attributes:
///    - Green color.
///    - Bold text.
///    - Underlined text.
/// 4. A fixed status text `[OK]` is appended after the styled message, in dark green color and italic text.
/// 5. The styled content is written to stdout.
///
/// # Panics
/// - The function panics if the `message` is empty.
/// - Panics on failure while attempting to execute terminal operations such as clearing
///   the screen or printing styled content. These operations may fail in environments where
///   stdout or terminal capabilities are restricted.
///
/// # Examples
/// ```rust
/// use your_namespace::ok_clear;
///
/// // Prints a styled success message without clearing the screen.
/// ok_clear("Operation completed successfully.", false);
///
/// // Clears the terminal and then prints the message.
/// ok_clear("Task executed correctly.", true);
/// ```
///
/// # Dependencies
/// This function uses the `crossterm` crate for terminal operations, including
/// - `execute` for executing terminal commands.
/// - `Clear` and `ClearType` for clearing the terminal.
/// - `MoveTo` for moving the cursor to a specific position.
/// - `PrintStyledContent` for printing styled text.
///
/// The `colored` crate is also used to apply styles like color, bold, underlined, and italic.
pub fn ok_clear(message: &str, clear: bool) {
    if message.is_empty() {
        panic!("Message is empty");
    }
    if clear {
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).expect("Failed to clear terminal");
    }
    let styled_message = format!("✔ {message}")
        .with(Color::Green)
        .bold()
        .underlined();
    execute!(
        stdout(),
        PrintStyledContent(styled_message),
        PrintStyledContent("  [OK]".with(Color::DarkGreen).italic())
    )
    .expect("Failed to print styled message");
    println!();
}
/// Downloads the content from the specified URI and saves it to a file with the given file name.
///
/// # Arguments
///
/// * `uri` - A string slice that holds the URI of the resource to download.
/// * `file_name` - A string slice that specifies the name of the file to save the downloaded content.
///
/// # Returns
///
/// * `Result<(), std::io::Error>` - Returns `Ok(())` if the download and file writing are successful;
///   returns an `Err` if an error occurs during file operations.
///
/// # Errors
///
/// This function will return an error if:
/// - The HTTP request to the specified URI fails or is unsuccessful.
/// - The response content cannot be read as bytes.
/// - There is an error while creating or writing to the file.
///
/// # Side Effects
///
/// - Logs a message using the `ok_clear` function to indicate the success of the download
///   and the elapsed time taken for the operation.
///
/// # Panics
///
/// This function will panic if:
/// - The HTTP GET request fails (i.e., if `reqwest::blocking::get(uri)` returns an error).
/// - The response content cannot be read as bytes (i.e., if `response.bytes()` fails).
///
/// # Examples
///
/// ```
/// use std::io;
///
/// fn main() -> Result<(), io::Error> {
///     let uri = "https://example.com/somefile.txt";
///     let file_name = "downloaded_file.txt";
///
///     ok_download(uri, file_name)?;
///
///     Ok(())
/// }
/// ```
pub fn ok_download(uri: &str, file_name: &str) -> Result<(), std::io::Error> {
    let now: Instant = Instant::now();
    let response: reqwest::blocking::Response = get(uri).expect("Request failed");
    let content = response.bytes().expect("Failed to read response bytes");

    let mut downloaded_file = File::create(file_name)?;
    downloaded_file.write_all(&content)?;

    let duration = now.elapsed();
    ok_clear(&format!("Downloaded {file_name} in {duration:?}"), false);
    Ok(())
}
