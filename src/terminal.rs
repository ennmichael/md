use crossterm::{
    cursor::{Hide, MoveTo},
    event::{self, Event, KeyCode, KeyModifiers},
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal, ErrorKind, QueueableCommand,
};
use std::io::{self, Stdout, Write};
use std::{error::Error, fmt::Display};
use terminal::{Clear, ClearType};

use crate::common::{Dimensions, Position};

#[derive(Debug)]
enum TerminalErrorKind {
    Crossterm(ErrorKind),
    Io(io::Error),
}

#[derive(Debug)]
pub struct TerminalError(TerminalErrorKind);

pub type TerminalResult<T> = Result<T, TerminalError>;

impl From<ErrorKind> for TerminalError {
    fn from(e: ErrorKind) -> Self {
        Self(TerminalErrorKind::Crossterm(e))
    }
}

impl From<io::Error> for TerminalError {
    fn from(e: io::Error) -> Self {
        Self(TerminalErrorKind::Io(e))
    }
}

impl Error for TerminalError {}

impl Display for TerminalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            TerminalErrorKind::Crossterm(e) => e.fmt(f),
            TerminalErrorKind::Io(e) => e.fmt(f),
        }
    }
}

pub struct AnsiColor(pub u8);

pub struct Style {
    pub foregound: Option<AnsiColor>,
    pub background: Option<AnsiColor>,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    pub character: char,
    pub control: bool,
    pub shift: bool,
}

impl From<char> for Key {
    fn from(character: char) -> Self {
        Self {
            character,
            control: false,
            shift: false,
        }
    }
}

pub enum TerminalEvent {
    Key(Key),
    Resize { width: u32, height: u32 },
}

pub struct TerminalCanvas {
    stdout: Stdout,
}

impl TerminalCanvas {
    pub fn dimensions(&self) -> TerminalResult<Dimensions> {
        let size = terminal::size()?;
        Ok(Dimensions {
            width: size.0 as usize,
            height: size.1 as usize,
        })
    }

    pub fn width(&self) -> TerminalResult<usize> {
        Ok(self.dimensions()?.width)
    }

    pub fn height(&self) -> TerminalResult<usize> {
        Ok(self.dimensions()?.height)
    }

    pub fn clear(&mut self) -> TerminalResult<()> {
        self.stdout.queue(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn set_style(&mut self, style: &Style) -> TerminalResult<()> {
        self.stdout.queue(ResetColor)?;
        if let Some(ref fg) = style.foregound {
            self.stdout
                .queue(SetForegroundColor(Color::AnsiValue(fg.0)))?;
        }
        if let Some(ref bg) = style.background {
            self.stdout
                .queue(SetBackgroundColor(Color::AnsiValue(bg.0)))?;
        }
        if style.bold {
            self.stdout.queue(SetAttribute(Attribute::Bold))?;
        }
        if style.italic {
            self.stdout.queue(SetAttribute(Attribute::Italic))?;
        }
        Ok(())
    }

    pub fn print_str(&mut self, s: &str) -> TerminalResult<()> {
        self.stdout.queue(Print(s))?;
        Ok(())
    }

    pub fn print(&mut self, pos: &Position, c: char) -> TerminalResult<()> {
        self.stdout
            .queue(MoveTo(pos.x as u16, pos.y as u16))?
            .queue(Print(c))?;
        Ok(())
    }

    pub fn flush(&mut self) -> TerminalResult<()> {
        self.stdout.flush()?;
        Ok(())
    }
}

pub struct TerminalEvents;

impl TerminalEvents {
    pub fn next_event(&self) -> TerminalResult<TerminalEvent> {
        loop {
            match event::read()? {
                Event::Key(key_event) => {
                    if let KeyCode::Char(c) = key_event.code {
                        return Ok(TerminalEvent::Key(Key {
                            character: c,
                            control: key_event.modifiers.intersects(KeyModifiers::CONTROL),
                            shift: key_event.modifiers.intersects(KeyModifiers::SHIFT),
                        }));
                    }
                }
                Event::Resize(w, h) => {
                    return Ok(TerminalEvent::Resize {
                        width: w as u32,
                        height: h as u32,
                    });
                }
                _ => {}
            }
        }
    }
}

pub fn start_in_raw_mode() -> TerminalResult<(TerminalCanvas, TerminalEvents)> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.queue(Hide)?.flush()?;
    start()
}

pub fn start() -> TerminalResult<(TerminalCanvas, TerminalEvents)> {
    Ok((
        TerminalCanvas {
            stdout: io::stdout(),
        },
        TerminalEvents,
    ))
}

pub fn exit() {
    terminal::disable_raw_mode().unwrap();
}
