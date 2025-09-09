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
