use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use fmt::Debug;

use crate::{
    markdown_parser::{Heading, Markdown, MarkdownElement, Style, StyledWord},
    terminal::{Style as TerminalStyle, TerminalCanvas, TerminalError},
};

use layout::LayoutElement;

mod layout;

pub enum RendererLine<'a> {
    Heading(Heading<'a>),
    Text(&'a [StyledWord<'a>]),
}

struct Renderer<'a> {
    canvas: TerminalCanvas,
    lines: Vec<RendererLine<'a>>,
}

#[derive(Debug)]
pub enum RendererError {
    TerminalError(TerminalError),
}

// XXX Temporary solution
pub fn render(terminal: &mut TerminalCanvas, markdown: &Markdown) {
    for element in markdown.elements.iter() {
        match element {
            MarkdownElement::Heading(_) => panic!(),
            MarkdownElement::Paragraph(words) => {
                let layout = layout::calculate_layout(terminal.width().unwrap(), words);
                for line in layout {
                    for layout_element in line.elements {
                        match layout_element {
                            LayoutElement::Word(word) => render_word(terminal, &word),
                            LayoutElement::Whitespace(n) => render_whitespace(terminal, n),
                        }
                    }
                }
            }
        }
    }
}

fn render_word(terminal: &mut TerminalCanvas, word: &StyledWord) {
    terminal.set_style(&to_terminal_style(&word.style)).unwrap();
    terminal.print_str(word.text).unwrap();
}

fn render_whitespace(terminal: &mut TerminalCanvas, n: usize) {
    for _ in 0..n {
        terminal.print_str(" ").unwrap();
    }
}

fn to_terminal_style(style: &Style) -> TerminalStyle {
    TerminalStyle {
        foregound: None,
        background: None,
        bold: style.bold,
        italic: style.italic,
    }
}

impl Display for RendererError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::TerminalError(e) => Display::fmt(&e, f),
        }
    }
}

impl Error for RendererError {}

impl From<TerminalError> for RendererError {
    fn from(e: TerminalError) -> Self {
        RendererError::TerminalError(e)
    }
}

pub type RendererResult<T> = std::result::Result<T, RendererError>;

impl<'a> Renderer<'a> {
    pub fn new(canvas: TerminalCanvas) -> Self {
        Self {
            canvas,
            lines: Vec::new(),
        }
    }

    pub fn load_markdown(&mut self, markdown: &Markdown<'a>) {
        // XXX Convert the markdown into `self.lines`
    }

    pub fn paint(&mut self) -> RendererResult<()> {
        self.canvas.clear()?;
        Ok(())
    }

    pub fn scroll_down(&mut self) {}

    pub fn scroll_up(&mut self) {}
}
