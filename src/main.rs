use std::io;
use std::path::PathBuf;
use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use croot::app::App;
use croot::{config, render, syntax};
use crossterm::{
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

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
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
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
        Some(Command::Completions { shell }) => {
            clap_complete::generate(shell, &mut Cli::command(), "croot", &mut std::io::stdout());
            return Ok(());
        }
        None => {}
    }

    let path = cli.path.canonicalize().unwrap_or_else(|_| cli.path.clone());

    if !path.is_dir() {
        eprintln!("error: '{}' is not a valid directory", cli.path.display());
        std::process::exit(1);
    }

    // Load config before terminal setup so we know whether to enable mouse
    let cfg = config::Config::load();
    render::colors::init(&cfg.colors);
    syntax::theme::init(&cfg.syntax);
    let mouse_enabled = cfg.mouse.enabled;

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if mouse_enabled {
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    } else {
        execute!(stdout, EnterAlternateScreen)?;
    }

    // Enable Kitty keyboard protocol so we can receive Super (Command) modifier
    let enhanced_keyboard = crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false);
    if enhanced_keyboard {
        execute!(
            stdout,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
    }

    // Enable bracketed paste so we can distinguish typed input from pasted text
    execute!(stdout, EnableBracketedPaste)?;

    // Query terminal for graphics protocol support (must happen before EventStream consumes stdin)
    #[cfg(feature = "image-preview")]
    let image_picker = ratatui_image::picker::Picker::from_query_stdio().ok();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Helper closure for terminal teardown (used on both success and error paths)
    let teardown = |terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
                    enhanced_kb: bool,
                    mouse: bool|
     -> anyhow::Result<()> {
        disable_raw_mode()?;
        let _ = execute!(terminal.backend_mut(), DisableBracketedPaste);
        if enhanced_kb {
            let _ = execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags);
        }
        if mouse {
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
        } else {
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        }
        terminal.show_cursor()?;
        Ok(())
    };

    // Run app
    let mut app = match App::new(
        path,
        enhanced_keyboard,
        cfg,
        #[cfg(feature = "image-preview")]
        image_picker,
    ) {
        Ok(a) => a,
        Err(e) => {
            let _ = teardown(&mut terminal, enhanced_keyboard, mouse_enabled);
            return Err(e);
        }
    };
    let result = app.run(&mut terminal).await;

    // Terminal teardown
    teardown(&mut terminal, enhanced_keyboard, mouse_enabled)?;

    result
}

fn handle_config(action: Option<ConfigAction>) -> anyhow::Result<()> {
    match action {
        None => {
            // Print resolved config
            let cfg = config::Config::load();
            print!("{}", cfg.to_toml_string()?);
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
            let parts =
                shell_words::split(&editor_str).unwrap_or_else(|_| vec![editor_str.clone()]);
            let cmd = parts.first().map_or("vi", |s| s.as_str());
            let status = process::Command::new(cmd)
                .args(&parts[1..])
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
    let _ = process::Command::new("brew").args(["update"]).status();

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
            eprintln!("error: 'brew' not found. To update croot manually:");
            eprintln!("  cargo install croot       # if installed via cargo");
            eprintln!("  brew install croot        # install Homebrew first: https://brew.sh");
            process::exit(1);
        }
        Err(e) => Err(e.into()),
    }
}
