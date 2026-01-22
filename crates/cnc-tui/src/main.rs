use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

mod app;
mod config;
mod render;

use app::App;
use config::Action;

#[derive(Parser)]
#[command(author, version, about = "CNC toolpath viewer")]
struct Args {
    #[arg(value_name = "FILE")]
    file: PathBuf,

    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = config::Config::load(args.config)?;
    let file_content = std::fs::read_to_string(&args.file)?;
    let file_lines: Vec<String> = file_content.lines().map(|line| line.to_string()).collect();
    let options = cnc_gcode::ParseOptions::with_ignore_missing(
        config.parser.ignore_missing_words.clone(),
    )
    .with_ignore_unknown_words(config.parser.ignore_unknown_words);
    let toolpath = cnc_gcode::parse_file_with_options(&args.file, options)?;
    let mut app = App::new(config, toolpath, args.file, file_lines);

    run(&mut app)
}

type TerminalBackend = CrosstermBackend<std::io::Stdout>;

fn run(app: &mut App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, app);

    restore_terminal(&mut terminal)?;
    result
}

fn restore_terminal(terminal: &mut Terminal<TerminalBackend>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_loop(terminal: &mut Terminal<TerminalBackend>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(16);
    let mut last_tick = std::time::Instant::now();
    loop {
        let now = std::time::Instant::now();
        let delta = now.saturating_duration_since(last_tick);
        last_tick = now;
        app.tick(delta);
        terminal.draw(|f| render::draw(f, app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if let Some(action) = app.config.keys.action_for(key) {
                    if action == Action::Quit {
                        break;
                    }
                    app.apply_action(action);
                }
            }
        }
    }

    Ok(())
}
