mod app;
mod cmux;
mod config;
mod git;
mod input;
mod layout;
mod preview;
mod render;
mod tree;
mod watcher;

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
    subcommand_negates_reqs = true
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
    /// Manage croot configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Open configuration file in editor
    Edit,
    /// Show configuration file path
    Path,
    /// Create default configuration file
    Init,
    /// Get a configuration value
    Get {
        /// Dotted key path (e.g. `tree.show_hidden`)
        key: String,
    },
    /// Set a configuration value
    Set {
        /// Dotted key path (e.g. `preview.split_ratio`)
        key: String,
        /// Value to set
        value: String,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Update) => return self_update(),
        Some(Command::Config { action }) => return handle_config(action),
        None => {}
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
    let enhanced_keyboard = crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false);
    if enhanced_keyboard {
        execute!(
            stdout,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let mut app = App::new(path, enhanced_keyboard)?;
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

fn handle_config(action: Option<ConfigAction>) -> anyhow::Result<()> {
    match action {
        None => {
            // Print resolved config
            let cfg = config::Config::load();
            print!("{}", cfg.to_toml_string());
        }
        Some(ConfigAction::Path) => {
            println!("{}", config::config_path().display());
        }
        Some(ConfigAction::Init) => {
            let path = config::config_path();
            if path.exists() {
                eprintln!("error: config file already exists at {}", path.display());
                process::exit(1);
            }
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, config::Config::default_toml_with_comments())?;
            println!("Created {}", path.display());
        }
        Some(ConfigAction::Edit) => {
            let path = config::config_path();
            // Init if file doesn't exist
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, config::Config::default_toml_with_comments())?;
                eprintln!("Created {}", path.display());
            }
            let cfg = config::Config::load();
            let editor_str = config::resolve_editor(&cfg);
            let mut parts = editor_str.split_whitespace();
            let cmd = parts.next().unwrap_or("vi");
            let status = process::Command::new(cmd)
                .args(parts)
                .arg(&path)
                .status();
            match status {
                Ok(s) if s.success() => {}
                Ok(s) => process::exit(s.code().unwrap_or(1)),
                Err(e) => {
                    eprintln!("error: failed to launch editor '{cmd}': {e}");
                    process::exit(1);
                }
            }
        }
        Some(ConfigAction::Get { key }) => match config::get_value(&key) {
            Ok(val) => println!("{val}"),
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(1);
            }
        },
        Some(ConfigAction::Set { key, value }) => {
            if let Err(e) = config::set_value(&key, &value) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
    Ok(())
}

fn self_update() -> anyhow::Result<()> {
    println!("Updating croot...");

    // Refresh tap to get latest formula
    let _ = process::Command::new("brew")
        .args(["update"])
        .status();

    let status = process::Command::new("brew")
        .args(["upgrade", "croot"])
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
            eprintln!("error: 'brew' not found in PATH. Install Homebrew via https://brew.sh");
            process::exit(1);
        }
        Err(e) => Err(e.into()),
    }
}
