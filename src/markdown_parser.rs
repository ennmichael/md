#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyledWord<'a> {
    pub text: &'a str,
    pub style: Style,
}

impl<'a> From<&'a str> for StyledWord<'a> {
    fn from(text: &'a str) -> Self {
        StyledWord {
            text,
            style: Default::default(),
        }
    }
}

pub enum MarkdownElement<'a> {
    Paragraph(Vec<StyledWord<'a>>),
}

pub type Result<T> = std::result::Result<T, ()>;

pub fn parse(text: &str) -> Result<Vec<MarkdownElement>> {
    Ok(vec![MarkdownElement::Paragraph(
        text.split_ascii_whitespace()
            .map(|text| StyledWord {
                text,
                style: Default::default(),
            })
            .collect(),
    )])
}
