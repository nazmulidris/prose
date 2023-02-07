pub type MarkdownText<'a> = Vec<MarkdownInline<'a>>;

#[derive(Clone, Debug, PartialEq)]
pub enum Markdown<'a> {
    Heading(HeadingLevel, MarkdownText<'a>),
    OrderedList(Vec<MarkdownText<'a>>),
    UnorderedList(Vec<MarkdownText<'a>>),
    Line(MarkdownText<'a>),
    Codeblock(&'a str, &'a str),
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HeadingLevel {
    Heading1 = 1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
}

impl From<usize> for HeadingLevel {
    fn from(size: usize) -> Self {
        match size {
            1 => HeadingLevel::Heading1,
            2 => HeadingLevel::Heading2,
            3 => HeadingLevel::Heading3,
            4 => HeadingLevel::Heading4,
            5 => HeadingLevel::Heading5,
            6 => HeadingLevel::Heading6,
            _ => HeadingLevel::Heading6,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MarkdownInline<'a> {
    Link((&'a str, &'a str)),
    Image((&'a str, &'a str)),
    InlineCode(&'a str),
    Bold(&'a str),
    BoldItalic(&'a str),
    Italic(&'a str),
    Plaintext(&'a str),
}
