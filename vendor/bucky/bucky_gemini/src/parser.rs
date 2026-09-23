//! This module contains a parser for Gemtext documents.
//! ## Usage
//! ```
//! # use bucky_gemini::parser::{GemtextNode, Parser};
//! # fn main() {
//! let doc = "# Hello from gemtext!\nShall we parse?";
//! let parser = Parser::default();
//! let lines = parser.parse(doc);
//! assert_eq!(lines[0], GemtextNode::H1("Hello from gemtext!"));
//! assert_eq!(lines[1], GemtextNode::Text("Shall we parse?"));
//! # }
//! ```
use std::{convert::TryFrom, fmt::Display};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Link<'a> {
    pub url: &'a str,
    pub display: Option<String>,
}

impl<'a> TryFrom<&'a str> for Link<'a> {
    type Error = &'static str;

    fn try_from(text: &'a str) -> Result<Self, Self::Error> {
        let mut split = text.split_whitespace();
        match split.next() {
            Some("=>" | "=:") => {}
            _ => return Err("Not a link"),
        }
        if let Some(url) = split.next() {
            let display = split.collect::<Vec<&str>>().join(" ");
            if display.is_empty() {
                Ok(Link { url, display: None })
            } else {
                Ok(Link {
                    url,
                    display: Some(display),
                })
            }
        } else {
            Err("Not a link")
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
/// A singular gemtext node.
pub enum GemtextNode<'a> {
    /// A pure text block. The string contained within is the entire line of text
    Text(&'a str),
    /// A link.
    ///
    /// A link is found by a line starting with the characters "=>" followed by a space
    ///
    /// The first string contained is the link itself and the second string is an optional
    /// descriptor
    Link(Link<'a>),
    /// A prompt (Spartan only)
    ///
    /// A prompt is a line starting with the characters "=:" followed by a space
    /// The first string is the link and any further characters make up the optional
    /// display string
    Prompt(Link<'a>),
    /// A H1 heading
    /// A heading starts with a singular # with a space following.
    /// The string contained is the text that follows the heading marker
    H1(&'a str),
    /// An H2 heading
    ///
    /// An H2 heading starts with the characters "##" with a space following.
    /// The string contained is the text that folllows the subheading marker.
    H2(&'a str),
    /// An H3 heading
    ///
    /// An H3 heading starts with the characters "###" with a space following.
    /// The string contained is the text that follows the subheading marker
    H3(&'a str),
    /// A list item
    ///
    /// A list item starts with the character "*".
    /// Unlike markdown, '-' is not allowed to start a list item
    ///
    /// The string contained is the text that follows the list item marker.
    ListItem(&'a str),
    /// A block quote
    ///
    /// A blockquote starts with the character ">".
    ///
    /// The string contained is the text that follows the blockquote marker
    Blockquote(String),
    /// A block of preformatted text.
    ///
    /// A preformatted text block starts with the characters "\`\`\`"
    ///
    /// The first string contained is the text within the preformatted text (newlines and all). The second string is an optional formatting tag for the preformatted text a la Markdown. It's worth noting that this is more clearly listed as an "alt text" as opposed to a formatting tag.
    Preformatted(String, Option<&'a str>),
}

impl<'a> Display for GemtextNode<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "{s}"),
            Self::Link(link) => match &link.display {
                Some(d) => write!(f, "=> {} {d}", link.url),
                None => write!(f, "=> {}", link.url),
            },
            Self::Prompt(link) => match &link.display {
                Some(d) => write!(f, "=: {} {d}", link.url),
                None => write!(f, "=: {}", link.url),
            },
            Self::H1(s) => write!(f, "# {s}"),
            Self::H2(s) => write!(f, "## {s}"),
            Self::H3(s) => write!(f, "### {s}"),
            Self::ListItem(s) => write!(f, "* {s}"),
            Self::Blockquote(s) => write!(f, "> {s}"),
            Self::Preformatted(s, None) => write!(f, "```\n{s}\n```"),
            Self::Preformatted(s, Some(d)) => {
                write!(f, "```{d}\n{s}\n```")
            }
        }
    }
}

impl<'a> GemtextNode<'a> {
    fn parse_link(text: &'a str) -> Self {
        if let Ok(link) = Link::try_from(text) {
            Self::Link(link)
        } else {
            Self::Text(text)
        }
    }

    fn parse_prompt(text: &'a str) -> Self {
        if let Ok(link) = Link::try_from(text) {
            Self::Prompt(link)
        } else {
            Self::Text(text)
        }
    }

    fn parse_heading(text: &'a str) -> Self {
        if let Some((h, s)) = text.split_once(' ') {
            match h {
                "#" => GemtextNode::H1(s),
                "##" => GemtextNode::H2(s),
                "###" => GemtextNode::H3(s),
                _ => GemtextNode::Text(text),
            }
        } else {
            GemtextNode::Text(text)
        }
    }

    fn parse_list_item(text: &'a str) -> Self {
        match text.split_once(' ') {
            Some((pre, s)) if pre == "*" => GemtextNode::ListItem(s),
            _ => GemtextNode::Text(text),
        }
    }

    fn parse_blockquote(text: &'a str) -> Self {
        match text.split_once(char::is_whitespace) {
            Some((prefix, suffix)) if prefix == ">" => GemtextNode::Blockquote(suffix.to_string()),
            _ => GemtextNode::Text(text),
        }
    }
}

enum State {
    Normal,
    Preformatted,
    Quote,
}

impl Default for State {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Default)]
/// A parser for Gemtext documents
pub struct Parser<'a> {
    state: State,
    preblk: String,
    pre_alt: Option<&'a str>,
    quoteblk: String,
    lines: Vec<GemtextNode<'a>>,
}

impl<'a> Parser<'a> {
    #[must_use]
    // Runs the loop over the full document
    pub fn parse(mut self, text: &'a str) -> Vec<GemtextNode<'a>> {
        for line in text.lines() {
            match self.state {
                State::Preformatted => self.parse_preformatted(line),
                State::Quote => self.parse_quote(line),
                State::Normal => self.parse_normal(line),
            }
        }
        self.lines
    }

    fn link(&mut self, line: &'a str) {
        self.lines.push(GemtextNode::parse_link(line));
    }

    fn prompt(&mut self, line: &'a str) {
        self.lines.push(GemtextNode::parse_prompt(line));
    }

    fn heading(&mut self, line: &'a str) {
        self.lines.push(GemtextNode::parse_heading(line));
    }

    fn list_item(&mut self, line: &'a str) {
        self.lines.push(GemtextNode::parse_list_item(line));
    }

    fn enter_preformatted(&mut self, line: &'a str) {
        self.state = State::Preformatted;
        if line.len() > 3 {
            self.pre_alt = Some(line[3..].trim());
        }
    }

    // Runs when the parser state is `State::Normal`
    fn parse_normal(&mut self, line: &'a str) {
        match line {
            s if s.starts_with("=>") => self.link(s),
            s if s.starts_with("=:") => self.prompt(s),
            s if s.starts_with('#') => self.heading(s),
            s if s.starts_with('*') => self.list_item(s),
            s if s.starts_with('>') => match GemtextNode::parse_blockquote(s) {
                GemtextNode::Blockquote(q) => {
                    self.quoteblk.push_str(&q);
                    self.quoteblk.push('\n');
                    self.state = State::Quote;
                }
                GemtextNode::Text(t) => self.lines.push(GemtextNode::Text(t)),
                _ => unreachable!(),
            },
            s if s.starts_with("```") => self.enter_preformatted(s),
            s => self.lines.push(GemtextNode::Text(s)),
        }
    }

    // Runs when the parser state is `State::Preformatted`
    fn parse_preformatted(&mut self, line: &str) {
        if line.starts_with("```") {
            self.lines.push(GemtextNode::Preformatted(
                self.preblk.trim_end().to_string(),
                self.pre_alt,
            ));
            self.state = State::Normal;
            self.pre_alt = None;
            self.preblk.clear();
        } else {
            self.preblk.push_str(line);
            self.preblk.push('\n');
        }
    }

    // Runs when the parser state is `State::Quote`
    fn parse_quote(&mut self, line: &'a str) {
        if line.starts_with('>') {
            match GemtextNode::parse_blockquote(line) {
                GemtextNode::Blockquote(s) => {
                    self.quoteblk.push_str(&s);
                    self.quoteblk.push('\n');
                }
                GemtextNode::Text(s) => {
                    self.lines.push(GemtextNode::Blockquote(
                        self.quoteblk.trim_end().to_string(),
                    ));
                    self.lines.push(GemtextNode::Text(s));
                    self.state = State::Normal;
                    self.quoteblk.clear();
                }
                _ => unreachable!(),
            }
            return;
        }
        self.lines.push(GemtextNode::Blockquote(
            self.quoteblk.trim_end().to_string(),
        ));
        self.state = State::Normal;
        self.quoteblk.clear();
        match line {
            s if s.starts_with("=>") => self.link(s),
            s if s.starts_with("=:") => self.prompt(s),
            s if s.starts_with('#') => self.heading(s),
            s if s.starts_with('*') => self.list_item(s),
            s if s.starts_with("```") => self.enter_preformatted(s),
            s => self.lines.push(GemtextNode::Text(s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GemtextNode, Link, Parser};

    #[test]
    fn parse_link() {
        let line = "=>   gemini://test.gmi Test line";
        let parsed = GemtextNode::parse_link(line);
        assert_eq!(
            parsed,
            GemtextNode::Link(Link {
                url: "gemini://test.gmi",
                display: Some(String::from("Test line")),
            })
        );
    }

    #[test]
    fn parse_link_nodisplay() {
        let line = "=> spartan://c.ca";
        let parsed = GemtextNode::parse_link(line);
        assert_eq!(
            parsed,
            GemtextNode::Link(Link {
                url: "spartan://c.ca",
                display: None,
            })
        );
    }

    #[test]
    fn parse_link_malformed() {
        let line = "=>gemini://c.ca";
        let parsed = GemtextNode::parse_link(line);
        assert_eq!(parsed, GemtextNode::Text(line))
    }

    #[test]
    fn parse_prompt() {
        let line = "=:   gemini://test.gmi/echo Test	echo";
        let parsed = GemtextNode::parse_prompt(line);
        assert_eq!(
            parsed,
            GemtextNode::Prompt(Link {
                url: "gemini://test.gmi/echo",
                display: Some(String::from("Test echo")),
            })
        );
    }

    #[test]
    fn parse_prompt_nodisplay() {
        let line = "=: spartan://c.ca/echo";
        let parsed = GemtextNode::parse_prompt(line);
        assert_eq!(
            parsed,
            GemtextNode::Prompt(Link {
                url: "spartan://c.ca/echo",
                display: None,
            })
        );
    }

    #[test]
    fn parse_h1() {
        let line = "# Hello World!";
        let parsed = GemtextNode::parse_heading(line);
        assert_eq!(parsed, GemtextNode::H1("Hello World!"));
    }

    #[test]
    fn parse_h2() {
        let line = "## Hello World!";
        let parsed = GemtextNode::parse_heading(line);
        assert_eq!(parsed, GemtextNode::H2("Hello World!"));
    }

    #[test]
    fn parse_h3() {
        let line = "### Hello World!";
        let parsed = GemtextNode::parse_heading(line);
        assert_eq!(parsed, GemtextNode::H3("Hello World!"));
    }

    #[test]
    fn parse_heading_malformed() {
        let line = "##Hello World!";
        let parsed = GemtextNode::parse_heading(line);
        assert_eq!(parsed, GemtextNode::Text(line));
    }

    #[test]
    fn parse_li() {
        let line = "* Item 1";
        let parsed = GemtextNode::parse_list_item(line);
        assert_eq!(parsed, GemtextNode::ListItem("Item 1"));
    }

    #[test]
    fn parse_li_bad_prefix() {
        let line = "** Item 2";
        let parsed = GemtextNode::parse_list_item(line);
        assert_eq!(parsed, GemtextNode::Text(line));
    }

    #[test]
    fn parse_li_nospace() {
        let line = "*Item3";
        let parsed = GemtextNode::parse_list_item(line);
        assert_eq!(parsed, GemtextNode::Text(line));
    }

    #[test]
    fn parse_quote() {
        let line = "> Don't Panic";
        let parsed = GemtextNode::parse_blockquote(line);
        assert_eq!(parsed, GemtextNode::Blockquote("Don't Panic".to_string()));
    }

    #[test]
    fn parse_quote_bad_prefix() {
        let line = ">> So long and thanks for all the fish";
        let parsed = GemtextNode::parse_blockquote(line);
        assert_eq!(parsed, GemtextNode::Text(line));
    }

    #[test]
    fn parse_quote_nospace() {
        let line = ">Oh no not again";
        let parsed = GemtextNode::parse_blockquote(line);
        assert_eq!(parsed, GemtextNode::Text(line));
    }

    #[test]
    fn parse_doc() {
        let doc = include_str!("test.gmi");
        let parsed = Parser::default().parse(doc);
        assert_eq!(parsed[0], GemtextNode::H1("A heading"));
        assert_eq!(
            parsed[1],
            GemtextNode::Preformatted(
                "# This should be preformatted\n> And this\n* This too\nAnd this".to_string(),
                Some("Ignore this"),
            )
        );
        assert_eq!(parsed[2], GemtextNode::Text("This is just text."));
        assert_eq!(
            parsed[3],
            GemtextNode::Blockquote("This is a single line blockquote".to_string())
        );
        assert_eq!(parsed[4], GemtextNode::ListItem("A list item"));
        assert_eq!(
            parsed[5],
            GemtextNode::Blockquote(
                "A blockquote spanning\nMultiple lines\nI can do this all day".to_string()
            )
        );
        assert_eq!(
            parsed[6],
            GemtextNode::Link(Link {
                url: "gemini://the.end",
                display: Some("This is the end".to_string()),
            })
        );
    }
}
