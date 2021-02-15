use std::fs;

use markdown_parser::{Markdown, MarkdownElement};
use terminal::TerminalCanvas;

mod terminal;
mod keybindings;
mod markdown_parser;
mod renderer;
mod common;

fn main() {
    let text = fs::read_to_string("sample.short.md").unwrap();
    let markdown = Markdown::parse(&text).unwrap();
    let (mut terminal_canvas, terminal_events) = terminal::start().unwrap();

    println!("{}", terminal_canvas.width().unwrap());
    renderer::render(&mut terminal_canvas, &markdown);

    terminal::exit();
}
