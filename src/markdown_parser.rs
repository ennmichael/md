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

pub enum HeadingSize {
    Small,
    Medium,
    Large,
}

pub struct Heading<'a> {
    words: Vec<StyledWord<'a>>,
    size: HeadingSize,
}

pub enum MarkdownElement<'a> {
    Heading(Heading<'a>),
    Paragraph(Vec<StyledWord<'a>>),
}

pub type Result<T> = std::result::Result<T, ()>;

pub struct Markdown<'a> {
    pub elements: Vec<MarkdownElement<'a>>,
}

impl<'a> Markdown<'a> {
    pub fn parse(text: &'a str) -> Result<Self> {
        Ok(Self {
            elements: vec![MarkdownElement::Paragraph(
                text.split_ascii_whitespace()
                    .map(|text| StyledWord {
                        text,
                        style: Default::default(),
                    })
                    .collect(),
            )],
        })
    }
}
