use crate::*;
use nom::{
    branch::*, bytes::complete::*, character::*, combinator::*, multi::*, sequence::*, IResult,
};

/// Main entry point for the MD parsing module.
pub fn render_markdown(md: &str) -> String {
    match parse_markdown(md) {
         Ok((_, m)) => translate(m),
         Err(_) => String::from("Sorry, this did not seem to work! Maybe your markdown was not well formed, have you hit [Enter] after your last line?"),
     }
}

pub mod constants {
    pub const HEADING_CHAR: char = '#';
    pub const SPACE_STR: &str = " ";
}

/// Skip rustfmt for this module: <https://stackoverflow.com/a/67289474/2085356>. It is cleaner
/// to write parsers w/out rustfmt reformatting them. This module is just to localize this directive
/// to rustfmt to suspend reformatting.
#[rustfmt::skip]
pub mod parser_impl {
    use super::*;

    pub fn parse_markdown(input: &str) -> IResult<&str, Vec<Markdown>> {
        many0(
            alt((
                map(parse_heading,
                    |(level, text)| Markdown::Heading(level, text)),
                map(parse_unordered_list,
                    Markdown::UnorderedList),
                map(parse_ordered_list,
                    Markdown::OrderedList),
                map(parse_code_block,
                    |(lang, body)| Markdown::Codeblock(lang, body)),
                map(parse_markdown_text_until_eol,
                    Markdown::Line),
            ))
        )(input)
    }

    pub fn parse_bold_italic(input: &str) -> IResult<&str, &str> {
        alt((
            delimited(tag("***"), is_not("***"), tag("***")),
            delimited(tag("___"), is_not("___"), tag("___")),
        ))(input)
    }

    pub fn parse_bold(input: &str) -> IResult<&str, &str> {
        alt((
            delimited(tag("**"), is_not("**"), tag("**")),
            delimited(tag("__"), is_not("__"), tag("__")),
        ))(input)
    }

    pub fn parse_italic(input: &str) -> IResult<&str, &str> {
        alt((
            delimited(tag("*"), is_not("*"), tag("*")),
            delimited(tag("_"), is_not("_"), tag("_"))
        ))(input)
    }

    pub fn parse_inline_code(input: &str) -> IResult<&str, &str> {
        delimited(tag("`"), is_not("`"), tag("`"))(input)
    }

    pub fn parse_link(i: &str) -> IResult<&str, (&str, &str)> {
        pair(
            delimited(tag("["), is_not("]"), tag("]")),
            delimited(tag("("), is_not(")"), tag(")")),
        )(i)
    }

    pub fn parse_image(i: &str) -> IResult<&str, (&str, &str)> {
        pair(
            delimited(tag("!["), is_not("]"), tag("]")),
            delimited(tag("("), is_not(")"), tag(")")),
        )(i)
    }

    // We want to match many things that are not any of our special tags but since we have no tools
    // available to match and consume in the negative case (without regex) we need to match against
    // our tags, then consume one char we repeat this until we run into one of our special
    // characters then we return this slice.
    pub fn parse_plaintext(i: &str) -> IResult<&str, &str> {
        recognize(many1(preceded(
            not(alt((tag("*"), tag("`"), tag("["), tag("!["), tag("\n")))),
            take(1u8),
        )))(i)
    }

    /// Parse chunks of markdown text that are in a single line.
    pub fn parse_markdown_inline(input: &str) -> IResult<&str, MarkdownInline> {
        alt((
            map(parse_italic, MarkdownInline::Italic),
            map(parse_bold, MarkdownInline::Bold),
            map(parse_bold_italic, MarkdownInline::BoldItalic),
            map(parse_inline_code, MarkdownInline::InlineCode),
            map(parse_image, MarkdownInline::Image),
            map(parse_link, MarkdownInline::Link),
            map(parse_plaintext, MarkdownInline::Plaintext),
        ))(input)
    }

    pub fn parse_markdown_text_until_eol(input: &str) -> IResult<&str, MarkdownText> {
        terminated(
            many0(parse_markdown_inline),
            tag("\n")
        )(input)
    }

    /// Matches one or more `#` chars.
    pub fn parse_heading_tag(input: &str) -> IResult<&str, HeadingLevel> {
        map(
            terminated(
                take_while1(|it| it == constants::HEADING_CHAR),
                tag(constants::SPACE_STR)
            ),
            |it: &str| HeadingLevel::from(it.len()),
        )(input)
    }

    /// This combines a tuple of the heading tag and the rest of the line.
    pub fn parse_heading(input: &str) -> IResult<&str, (HeadingLevel, MarkdownText)> {
        tuple(
            (parse_heading_tag, parse_markdown_text_until_eol)
        )(input)
    }

    pub fn parse_unordered_list_tag(i: &str) -> IResult<&str, &str> {
        terminated(tag("-"), tag(" "))(i)
    }

    pub fn parse_unordered_list_element(i: &str) -> IResult<&str, MarkdownText> {
        preceded(parse_unordered_list_tag, parse_markdown_text_until_eol)(i)
    }

    pub fn parse_unordered_list(i: &str) -> IResult<&str, Vec<MarkdownText>> {
        many1(parse_unordered_list_element)(i)
    }

    pub fn parse_ordered_list_tag(i: &str) -> IResult<&str, &str> {
        terminated(
            terminated(take_while1(|d| is_digit(d as u8)), tag(".")),
            tag(" "),
        )(i)
    }

    pub fn parse_ordered_list_element(i: &str) -> IResult<&str, MarkdownText> {
        preceded(parse_ordered_list_tag, parse_markdown_text_until_eol)(i)
    }

    pub fn parse_ordered_list(i: &str) -> IResult<&str, Vec<MarkdownText>> {
        many1(parse_ordered_list_element)(i)
    }

    pub fn parse_code_block(input: &str) -> IResult<&str, (/* lang */ &str, /* body */ &str)> {
        tuple(
            (parse_code_block_lang, parse_code_block_body)
        )(input)
    }

    pub fn parse_code_block_body(input: &str) -> IResult<&str, &str> {
        delimited(tag("\n"), is_not("```"), tag("```"))(input)
    }

    pub fn parse_code_block_lang(input: &str) -> IResult<&str, &str> {
        alt((
            preceded(tag("```"), parse_plaintext),
            map(tag("```"), |_| "__UNKNOWN_LANGUAGE__"),
        ))(input)
    }

}
pub use parser_impl::*;

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{error::Error, error::ErrorKind, Err as NomErr};

    #[test]
    fn test_parse_italic() {
        assert_eq!(parse_italic("*here is italic*"), Ok(("", "here is italic")));

        assert_eq!(parse_italic("_here is italic_"), Ok(("", "here is italic")));

        assert_eq!(
            parse_italic("*here is italic"),
            Err(NomErr::Error(Error {
                input: "*here is italic",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_italic("here is italic*"),
            Err(NomErr::Error(Error {
                input: "here is italic*",
                code: ErrorKind::Tag,
            }))
        );

        assert_eq!(
            parse_italic("here is italic"),
            Err(NomErr::Error(Error {
                input: "here is italic",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_italic("*"),
            Err(NomErr::Error(Error {
                input: "*",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_italic("**"),
            Err(NomErr::Error(Error {
                input: "**",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_italic(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_italic("**we are doing bold**"),
            Err(NomErr::Error(Error {
                input: "**we are doing bold**",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_bold_italic() {
        assert_eq!(
            parse_bold_italic("***here is bitalic***"),
            Ok(("", "here is bitalic"))
        );

        assert_eq!(
            parse_bold("***here is bitalic"),
            Err(NomErr::Error(Error {
                input: "***here is bitalic",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("here is bitalic***"),
            Err(NomErr::Error(Error {
                input: "here is bitalic***",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold_italic("___here is bitalic___"),
            Ok(("", "here is bitalic"))
        );

        assert_eq!(
            parse_bold_italic("___here is bitalic"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold_italic("here is bitalic___"),
            Err(NomErr::Error(Error {
                input: "here is bitalic___",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_bold() {
        assert_eq!(parse_bold("**here is bold**"), Ok(("", "here is bold")));

        assert_eq!(parse_bold("__here is bold__"), Ok(("", "here is bold")));

        assert_eq!(
            parse_bold("**here is bold"),
            Err(NomErr::Error(Error {
                input: "**here is bold",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("here is bold**"),
            Err(NomErr::Error(Error {
                input: "here is bold**",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("here is bold"),
            Err(NomErr::Error(Error {
                input: "here is bold",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("****"),
            Err(NomErr::Error(Error {
                input: "****",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("**"),
            Err(NomErr::Error(Error {
                input: "**",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("*"),
            Err(NomErr::Error(Error {
                input: "*",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );

        assert_eq!(
            parse_bold("*this is italic*"),
            Err(NomErr::Error(Error {
                input: "*this is italic*",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_inline_code() {
        assert_eq!(parse_bold("**here is bold**\n"), Ok(("\n", "here is bold")));
        assert_eq!(
            parse_inline_code("`here is code"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_inline_code("here is code`"),
            Err(NomErr::Error(Error {
                input: "here is code`",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_inline_code("``"),
            Err(NomErr::Error(Error {
                input: "`",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq!(
            parse_inline_code("`"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::IsNot
            }))
        );
        assert_eq!(
            parse_inline_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_link() {
        assert_eq!(
            parse_link("[title](https://www.example.com)"),
            Ok(("", ("title", "https://www.example.com")))
        );
        assert_eq!(
            parse_inline_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_image() {
        assert_eq!(
            parse_image("![alt text](image.jpg)"),
            Ok(("", ("alt text", "image.jpg")))
        );
        assert_eq!(
            parse_inline_code(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_plaintext() {
        assert_eq!(parse_plaintext("1234567890"), Ok(("", "1234567890")));
        assert_eq!(parse_plaintext("oh my gosh!"), Ok(("", "oh my gosh!")));
        assert_eq!(parse_plaintext("oh my gosh!["), Ok(("![", "oh my gosh")));
        assert_eq!(parse_plaintext("oh my gosh!*"), Ok(("*", "oh my gosh!")));
        assert_eq!(
            parse_plaintext("*bold baby bold*"),
            Err(NomErr::Error(Error {
                input: "*bold baby bold*",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("[link baby](and then somewhat)"),
            Err(NomErr::Error(Error {
                input: "[link baby](and then somewhat)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("`codeblock for bums`"),
            Err(NomErr::Error(Error {
                input: "`codeblock for bums`",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("![ but wait theres more](jk)"),
            Err(NomErr::Error(Error {
                input: "![ but wait theres more](jk)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("here is plaintext"),
            Ok(("", "here is plaintext"))
        );
        assert_eq!(
            parse_plaintext("here is plaintext!"),
            Ok(("", "here is plaintext!"))
        );
        assert_eq!(
            parse_plaintext("here is plaintext![image starting"),
            Ok(("![image starting", "here is plaintext"))
        );
        assert_eq!(
            parse_plaintext("here is plaintext\n"),
            Ok(("\n", "here is plaintext"))
        );
        assert_eq!(
            parse_plaintext("*here is italic*"),
            Err(NomErr::Error(Error {
                input: "*here is italic*",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("**here is bold**"),
            Err(NomErr::Error(Error {
                input: "**here is bold**",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("`here is code`"),
            Err(NomErr::Error(Error {
                input: "`here is code`",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("[title](https://www.example.com)"),
            Err(NomErr::Error(Error {
                input: "[title](https://www.example.com)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext("![alt text](image.jpg)"),
            Err(NomErr::Error(Error {
                input: "![alt text](image.jpg)",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_plaintext(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Eof
            }))
        );
    }

    #[test]
    fn test_parse_markdown_inline() {
        assert_eq!(
            parse_markdown_inline("*here is italic*"),
            Ok(("", MarkdownInline::Italic("here is italic")))
        );
        assert_eq!(
            parse_markdown_inline("**here is bold**"),
            Ok(("", MarkdownInline::Bold("here is bold")))
        );
        assert_eq!(
            parse_markdown_inline("`here is code`"),
            Ok(("", MarkdownInline::InlineCode("here is code")))
        );
        assert_eq!(
            parse_markdown_inline("[title](https://www.example.com)"),
            Ok((
                "",
                (MarkdownInline::Link(("title", "https://www.example.com")))
            ))
        );
        assert_eq!(
            parse_markdown_inline("![alt text](image.jpg)"),
            Ok(("", (MarkdownInline::Image(("alt text", "image.jpg")))))
        );
        assert_eq!(
            parse_markdown_inline("here is plaintext!"),
            Ok(("", MarkdownInline::Plaintext("here is plaintext!")))
        );
        assert_eq!(
            parse_markdown_inline("here is some plaintext *but what if we italicize?"),
            Ok((
                "*but what if we italicize?",
                MarkdownInline::Plaintext("here is some plaintext ")
            ))
        );
        assert_eq!(
            parse_markdown_inline("here is some plaintext \n*but what if we italicize?"),
            Ok((
                "\n*but what if we italicize?",
                MarkdownInline::Plaintext("here is some plaintext ")
            ))
        );
        assert_eq!(
            parse_markdown_inline("\n"),
            Err(NomErr::Error(Error {
                input: "\n",
                code: ErrorKind::Not
            }))
        );
        assert_eq!(
            parse_markdown_inline(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Eof
            }))
        );
    }

    #[test]
    fn test_parse_markdown_text() {
        assert_eq!(parse_markdown_text_until_eol("\n"), Ok(("", vec![])));
        assert_eq!(
            parse_markdown_text_until_eol("here is some plaintext\n"),
            Ok((
                "",
                vec![MarkdownInline::Plaintext("here is some plaintext")]
            ))
        );
        assert_eq!(
            parse_markdown_text_until_eol("here is some plaintext *but what if we italicize?*\n"),
            Ok((
                "",
                vec![
                    MarkdownInline::Plaintext("here is some plaintext "),
                    MarkdownInline::Italic("but what if we italicize?"),
                ]
            ))
        );
        assert_eq!(
            parse_markdown_text_until_eol("here is some plaintext *but what if we italicize?* I guess it doesn't **matter** in my `code`\n"),
            Ok(("", vec![
                MarkdownInline::Plaintext("here is some plaintext "),
                MarkdownInline::Italic("but what if we italicize?"),
                MarkdownInline::Plaintext(" I guess it doesn't "),
                MarkdownInline::Bold("matter"),
                MarkdownInline::Plaintext(" in my "),
                MarkdownInline::InlineCode("code"),
            ]))
        );
        assert_eq!(
            parse_markdown_text_until_eol("here is some plaintext *but what if we italicize?*\n"),
            Ok((
                "",
                vec![
                    MarkdownInline::Plaintext("here is some plaintext "),
                    MarkdownInline::Italic("but what if we italicize?"),
                ]
            ))
        );
        assert_eq!(
            parse_markdown_text_until_eol("here is some plaintext *but what if we italicize?"),
            Err(NomErr::Error(Error {
                input: "*but what if we italicize?",
                code: ErrorKind::Tag
            })) // Ok(("*but what if we italicize?", vec![MarkdownInline::Plaintext(String::from("here is some plaintext "))]))
        );
    }

    #[test]
    fn test_parse_header_tag() {
        assert_eq!(parse_heading_tag("# "), Ok(("", 1.into())));
        assert_eq!(parse_heading_tag("### "), Ok(("", 3.into())));
        assert_eq!(parse_heading_tag("# h1"), Ok(("h1", 1.into())));
        assert_eq!(parse_heading_tag("# h1"), Ok(("h1", 1.into())));
        assert_eq!(
            parse_heading_tag(" "),
            Err(NomErr::Error(Error {
                input: " ",
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_heading_tag("#"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_header() {
        assert_eq!(
            parse_heading("# h1\n"),
            Ok(("", (1.into(), vec![MarkdownInline::Plaintext("h1")])))
        );
        assert_eq!(
            parse_heading("## h2\n"),
            Ok(("", (2.into(), vec![MarkdownInline::Plaintext("h2")])))
        );
        assert_eq!(
            parse_heading("###  h3\n"),
            Ok(("", (3.into(), vec![MarkdownInline::Plaintext(" h3")])))
        );
        assert_eq!(
            parse_heading("###h3"),
            Err(NomErr::Error(Error {
                input: "h3",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_heading("###"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_heading(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_heading("#"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(parse_heading("# \n"), Ok(("", (1.into(), vec![]))));
        assert_eq!(
            parse_heading("# test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list_tag() {
        assert_eq!(parse_unordered_list_tag("- "), Ok(("", "-")));
        assert_eq!(
            parse_unordered_list_tag("- and some more"),
            Ok(("and some more", "-"))
        );
        assert_eq!(
            parse_unordered_list_tag("-"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag("-and some more"),
            Err(NomErr::Error(Error {
                input: "and some more",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag("--"),
            Err(NomErr::Error(Error {
                input: "-",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_tag(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list_element() {
        assert_eq!(
            parse_unordered_list_element("- this is an element\n"),
            Ok(("", vec![MarkdownInline::Plaintext("this is an element")]))
        );
        assert_eq!(
            parse_unordered_list_element(
                r#"- this is an element
- this is another element
"#
            ),
            Ok((
                "- this is another element\n",
                vec![MarkdownInline::Plaintext("this is an element")]
            ))
        );
        assert_eq!(
            parse_unordered_list_element(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(parse_unordered_list_element("- \n"), Ok(("", vec![])));
        assert_eq!(
            parse_unordered_list_element("- "),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_element("- test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list_element("-"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_unordered_list() {
        assert_eq!(
            parse_unordered_list("- this is an element"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_unordered_list("- this is an element\n"),
            Ok((
                "",
                vec![vec![MarkdownInline::Plaintext("this is an element")]]
            ))
        );
        assert_eq!(
            parse_unordered_list(
                r#"- this is an element
- here is another
"#
            ),
            Ok((
                "",
                vec![
                    vec![MarkdownInline::Plaintext("this is an element")],
                    vec![MarkdownInline::Plaintext("here is another")]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_ordered_list_tag() {
        assert_eq!(parse_ordered_list_tag("1. "), Ok(("", "1")));
        assert_eq!(parse_ordered_list_tag("1234567. "), Ok(("", "1234567")));
        assert_eq!(
            parse_ordered_list_tag("3. and some more"),
            Ok(("and some more", "3"))
        );
        assert_eq!(
            parse_ordered_list_tag("1"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag("1.and some more"),
            Err(NomErr::Error(Error {
                input: "and some more",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag("1111."),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_tag(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::TakeWhile1
            }))
        );
    }

    #[test]
    fn test_parse_ordered_list_element() {
        assert_eq!(
            parse_ordered_list_element("1. this is an element\n"),
            Ok(("", vec![MarkdownInline::Plaintext("this is an element")]))
        );
        assert_eq!(
            parse_ordered_list_element(
                r#"1. this is an element
1. here is another
"#
            ),
            Ok((
                "1. here is another\n",
                vec![MarkdownInline::Plaintext("this is an element")]
            ))
        );
        assert_eq!(
            parse_ordered_list_element(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(
            parse_ordered_list_element(""),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::TakeWhile1
            }))
        );
        assert_eq!(parse_ordered_list_element("1. \n"), Ok(("", vec![])));
        assert_eq!(
            parse_ordered_list_element("1. test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_element("1. "),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list_element("1."),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_parse_ordered_list() {
        assert_eq!(
            parse_ordered_list("1. this is an element\n"),
            Ok((
                "",
                vec![vec![MarkdownInline::Plaintext("this is an element")]]
            ))
        );
        assert_eq!(
            parse_ordered_list("1. test"),
            Err(NomErr::Error(Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            parse_ordered_list(
                r#"1. this is an element
2. here is another
"#
            ),
            Ok((
                "",
                vec![
                    vec!(MarkdownInline::Plaintext("this is an element")),
                    vec![MarkdownInline::Plaintext("here is another")]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_codeblock() {
        assert_eq!(
            parse_code_block(
                r#"```bash
pip install foobar
```"#
            ),
            Ok((
                "",
                (
                    "bash",
                    r#"pip install foobar
"#
                )
            ))
        );
        assert_eq!(
            parse_code_block(
                r#"```python
import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
```"#
            ),
            Ok((
                "",
                (
                    "python",
                    r#"import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
"#
                )
            ))
        );
        // assert_eq!(
        // 	parse_code_block("```bash\n pip `install` foobar\n```"),
        // 	Ok(("", "bash\n pip `install` foobar\n"))
        // );
    }

    #[test]
    fn test_parse_codeblock_no_language() {
        assert_eq!(
            parse_code_block(
                r#"```
pip install foobar
```"#
            ),
            Ok((
                "",
                (
                    "__UNKNOWN_LANGUAGE__",
                    r#"pip install foobar
"#
                )
            ))
        );
    }

    #[test]
    fn test_parse_markdown() {
        assert_eq!(
            parse_markdown(
                r#"# Foobar

Foobar is a Python library for dealing with word pluralization.

```bash
pip install foobar
```
## Installation

Use the package manager [pip](https://pip.pypa.io/en/stable/) to install foobar.
```python
import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
```"#
            ),
            Ok((
                "",
                vec![
                    Markdown::Heading(1.into(), vec![MarkdownInline::Plaintext("Foobar")]),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![MarkdownInline::Plaintext(
                        "Foobar is a Python library for dealing with word pluralization."
                    )]),
                    Markdown::Line(vec![]),
                    Markdown::Codeblock("bash", "pip install foobar\n"),
                    Markdown::Line(vec![]),
                    Markdown::Heading(
                        HeadingLevel::Heading2,
                        vec![MarkdownInline::Plaintext("Installation")]
                    ),
                    Markdown::Line(vec![]),
                    Markdown::Line(vec![
                        MarkdownInline::Plaintext("Use the package manager "),
                        MarkdownInline::Link(("pip", "https://pip.pypa.io/en/stable/")),
                        MarkdownInline::Plaintext(" to install foobar."),
                    ]),
                    Markdown::Codeblock(
                        "python",
                        r#"import foobar

foobar.pluralize('word') # returns 'words'
foobar.pluralize('goose') # returns 'geese'
foobar.singularize('phenomena') # returns 'phenomenon'
"#
                    ),
                ]
            ))
        )
    }
}
