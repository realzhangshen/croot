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

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;

#[derive(Parser)]
#[command(
    name = "croot",
    version,
    about = "A lightweight terminal file tree sidebar"
)]
struct Cli {
    /// Directory to browse (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let path = cli.path.canonicalize().unwrap_or_else(|_| cli.path.clone());

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let mut app = App::new(path)?;
    let result = app.run(&mut terminal).await;

    // Terminal teardown
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
