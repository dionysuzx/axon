use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{self, Stylize},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};

use crate::notes;

pub fn run() -> Result<(), crate::error::CliError> {
    if let Err(e) = enter_tui() {
        return Err(crate::error::CliError {
            code: 1,
            message: format!("TUI error: {e}"),
        });
    }
    Ok(())
}

fn enter_tui() -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;

    let result = event_loop(&mut stdout);

    // Always restore terminal
    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result
}

fn event_loop(stdout: &mut io::Stdout) -> io::Result<()> {
    draw_screen(stdout)?;

    loop {
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers,
                ..
            }) if !modifiers.contains(KeyModifiers::CONTROL) => {
                // Leave TUI, open editor, re-enter TUI
                stdout.execute(cursor::Show)?;
                stdout.execute(terminal::LeaveAlternateScreen)?;
                terminal::disable_raw_mode()?;

                let _ = notes::open_daily();

                terminal::enable_raw_mode()?;
                stdout.execute(terminal::EnterAlternateScreen)?;
                stdout.execute(cursor::Hide)?;
                draw_screen(stdout)?;
            }
            Event::Resize(_, _) => {
                draw_screen(stdout)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn draw_screen(stdout: &mut io::Stdout) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;

    stdout.queue(terminal::Clear(ClearType::All))?;

    // Title centered near top
    let title = "axon";
    let title_col = cols.saturating_sub(title.len() as u16) / 2;
    stdout.queue(cursor::MoveTo(title_col, 1))?;
    stdout.queue(style::PrintStyledContent(title.bold()))?;

    // Keybinding legend at bottom
    let legend_row = rows.saturating_sub(2);
    stdout.queue(cursor::MoveTo(2, legend_row))?;
    stdout.queue(style::PrintStyledContent("d".bold().cyan()))?;
    stdout.queue(style::PrintStyledContent(" daily  ".stylize()))?;
    stdout.queue(style::PrintStyledContent("q".bold().cyan()))?;
    stdout.queue(style::PrintStyledContent(" quit".stylize()))?;

    stdout.flush()?;
    Ok(())
}
