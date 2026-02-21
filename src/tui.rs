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

/// Shell out to fzf with the note list. Returns:
/// - Ok(Some(name)) if user selected/typed something
/// - Ok(None) if user escaped/cancelled
fn fzf_pick(files: &[String]) -> io::Result<Option<String>> {
    use std::process::{Command, Stdio};

    let input = files.join("\n");
    let mut child = Command::new("fzf")
        .args([
            "--print-query",
            "--header",
            "enter: open / select  esc: cancel",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write as _;
        let _ = stdin.write_all(input.as_bytes());
    }

    let output = child.wait_with_output()?;

    // fzf exit 130 = esc/ctrl-c, 1 = no match (but --print-query still prints query)
    let text = String::from_utf8_lossy(&output.stdout);
    let mut lines = text.lines();
    let query = lines.next().unwrap_or("").trim().to_string();
    let selection = lines.next().unwrap_or("").trim().to_string();

    if output.status.code() == Some(130) {
        return Ok(None);
    }

    // If user selected an existing match, prefer that
    if !selection.is_empty() {
        return Ok(Some(selection));
    }
    // Otherwise use the raw query (no match — will create)
    if !query.is_empty() {
        return Ok(Some(query));
    }

    Ok(None)
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

/// Suspend our TUI state, run a closure, restore.
/// Stays on the alternate screen to avoid flicker — child programs
/// (fzf, yazi) manage their own alternate screen internally.
fn shell_out(stdout: &mut io::Stdout, f: impl FnOnce()) -> io::Result<()> {
    stdout.execute(cursor::Show)?;
    terminal::disable_raw_mode()?;

    f();

    terminal::enable_raw_mode()?;
    stdout.execute(cursor::Hide)?;
    Ok(())
}

fn refresh_after_shell(stdout: &mut io::Stdout, files: &mut Vec<String>, selected: &mut usize) -> io::Result<()> {
    *files = notes::list_notes();
    if *selected >= files.len() {
        *selected = files.len().saturating_sub(1);
    }
    draw_screen(stdout, files, *selected)
}

fn event_loop(stdout: &mut io::Stdout) -> io::Result<()> {
    let mut files = notes::list_notes();
    let mut selected: usize = 0;

    draw_screen(stdout, &files, selected)?;

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

            // Fuzzy search via fzf
            Event::Key(KeyEvent {
                code: KeyCode::Char('/'),
                ..
            }) => {
                let pick = {
                    let f = files.clone();
                    let mut result = None;
                    shell_out(stdout, || {
                        result = fzf_pick(&f).ok().flatten();
                    })?;
                    result
                };

                if let Some(name) = pick {
                    if files.contains(&name) {
                        shell_out(stdout, || {
                            let _ = notes::open_note(&name);
                        })?;
                    } else {
                        shell_out(stdout, || {
                            let _ = notes::create_and_open_note(&name);
                        })?;
                    }
                }

                refresh_after_shell(stdout, &mut files, &mut selected)?;
            }

            // Navigation
            Event::Key(KeyEvent {
                code: KeyCode::Char('j') | KeyCode::Down,
                ..
            }) => {
                if !files.is_empty() && selected < files.len() - 1 {
                    selected += 1;
                    draw_screen(stdout, &files, selected)?;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k') | KeyCode::Up,
                ..
            }) => {
                if selected > 0 {
                    selected = selected.saturating_sub(1);
                    draw_screen(stdout, &files, selected)?;
                }
            }

            // Open selected note
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                if let Some(filename) = files.get(selected).cloned() {
                    shell_out(stdout, || {
                        let _ = notes::open_note(&filename);
                    })?;
                    refresh_after_shell(stdout, &mut files, &mut selected)?;
                }
            }

            // Shortcut keys
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers,
                ..
            }) if !modifiers.contains(KeyModifiers::CONTROL) => {
                shell_out(stdout, || { let _ = notes::open_daily(); })?;
                refresh_after_shell(stdout, &mut files, &mut selected)?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('w'),
                modifiers,
                ..
            }) if !modifiers.contains(KeyModifiers::CONTROL) => {
                shell_out(stdout, || { let _ = notes::open_weekly(); })?;
                refresh_after_shell(stdout, &mut files, &mut selected)?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('m'),
                modifiers,
                ..
            }) if !modifiers.contains(KeyModifiers::CONTROL) => {
                shell_out(stdout, || { let _ = notes::open_monthly(); })?;
                refresh_after_shell(stdout, &mut files, &mut selected)?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                modifiers,
                ..
            }) if !modifiers.contains(KeyModifiers::CONTROL) => {
                shell_out(stdout, || { let _ = notes::open_scratch(); })?;
                refresh_after_shell(stdout, &mut files, &mut selected)?;
            }

            Event::Resize(_, _) => {
                draw_screen(stdout, &files, selected)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn draw_screen(
    stdout: &mut io::Stdout,
    files: &[String],
    selected: usize,
) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;

    stdout.queue(terminal::Clear(ClearType::All))?;

    // Title centered near top
    let title = "axon";
    let title_col = cols.saturating_sub(title.len() as u16) / 2;
    stdout.queue(cursor::MoveTo(title_col, 1))?;
    stdout.queue(style::PrintStyledContent(title.bold()))?;

    // File list
    let list_start_row: u16 = 3;
    let legend_row = rows.saturating_sub(2);
    let available = legend_row.saturating_sub(list_start_row) as usize;

    if !files.is_empty() && available > 0 {
        let half = available / 2;
        let scroll_offset = if selected <= half {
            0
        } else if selected + half >= files.len() {
            files.len().saturating_sub(available)
        } else {
            selected - half
        };

        let end = files.len().min(scroll_offset + available);
        let pad: u16 = 4;

        for (i, name) in files[scroll_offset..end].iter().enumerate() {
            let abs_idx = scroll_offset + i;
            let row = list_start_row + i as u16;
            stdout.queue(cursor::MoveTo(pad, row))?;

            let max_width = (cols.saturating_sub(pad + 2)) as usize;
            let display: &str = if name.len() > max_width {
                &name[..max_width]
            } else {
                name
            };

            if abs_idx == selected {
                stdout.queue(style::PrintStyledContent(display.bold().cyan()))?;
            } else {
                stdout.queue(style::PrintStyledContent(display.stylize()))?;
            }
        }
    }

    // Legend
    stdout.queue(cursor::MoveTo(2, legend_row))?;
    stdout.queue(style::PrintStyledContent("/".bold().cyan()))?;
    stdout.queue(style::PrintStyledContent(" search  ".stylize()))?;
    stdout.queue(style::PrintStyledContent("d".bold().cyan()))?;
    stdout.queue(style::PrintStyledContent(" daily  ".stylize()))?;
    stdout.queue(style::PrintStyledContent("w".bold().cyan()))?;
    stdout.queue(style::PrintStyledContent(" weekly  ".stylize()))?;
    stdout.queue(style::PrintStyledContent("m".bold().cyan()))?;
    stdout.queue(style::PrintStyledContent(" monthly  ".stylize()))?;
    stdout.queue(style::PrintStyledContent("s".bold().cyan()))?;
    stdout.queue(style::PrintStyledContent(" scratch  ".stylize()))?;
    stdout.queue(style::PrintStyledContent("q".bold().cyan()))?;
    stdout.queue(style::PrintStyledContent(" quit".stylize()))?;

    stdout.flush()?;
    Ok(())
}
