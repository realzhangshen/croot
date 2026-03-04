mod app;
mod cmux;
mod config;
mod git;
mod input;
mod preview;
mod render;
mod tree;

use std::io;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;

#[derive(Parser)]
#[command(
    name = "croot",
    version,
    about = "A lightweight terminal file tree sidebar",
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
)]
struct Cli {
    /// Directory to browse (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Update croot to the latest version
    Update,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(Command::Update) = cli.command {
        return self_update();
    }

    let path = cli.path.canonicalize().unwrap_or_else(|_| cli.path.clone());

    if !path.is_dir() {
        eprintln!("error: '{}' is not a valid directory", cli.path.display());
        std::process::exit(1);
    }

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Enable Kitty keyboard protocol so we can receive Super (Command) modifier
    let enhanced_keyboard = crossterm::terminal::supports_keyboard_enhancement()
        .unwrap_or(false);
    if enhanced_keyboard {
        execute!(
            stdout,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let mut app = App::new(path)?;
    let result = app.run(&mut terminal).await;

    // Terminal teardown
    if enhanced_keyboard {
        execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags)?;
    }
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn self_update() -> anyhow::Result<()> {
    println!("Updating croot...");

    let status = process::Command::new("cargo")
        .args(["install", "croot", "--force"])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("croot updated successfully.");
            Ok(())
        }
        Ok(s) => {
            process::exit(s.code().unwrap_or(1));
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!("error: 'cargo' not found in PATH. Install Rust via https://rustup.rs");
            process::exit(1);
        }
        Err(e) => {
            Err(e.into())
        }
    }
}
