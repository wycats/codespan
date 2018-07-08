use std::borrow::Cow;
use std::fmt::{self, Arguments, Display};
use std::io;
use style::Attributes;
use termcolor::WriteColor;
use Style;

pub enum FormattedText<'a> {
    String(Cow<'a, str>),
    Args(Arguments<'a>),
}

impl<'a> Display for FormattedText<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FormattedText::String(string) => write!(f, "{}", string),
            FormattedText::Args(args) => write!(f, "{}", args),
        }
    }
}

pub struct TextNode<'a> {
    pub(crate) data: FormattedText<'a>,
    pub(crate) style: Style,
}

pub struct LineNode<'a> {
    pub(crate) parts: Vec<TextNode<'a>>,
}

impl<'a> LineNode<'a> {
    pub fn build() -> LineNodeBuilder<'a> {
        LineNodeBuilder {
            parts: vec![],
            style: Style::empty(),
        }
    }
}

#[must_use]
pub struct LineNodeBuilder<'a> {
    parts: Vec<TextNode<'a>>,
    style: Style,
}

impl<'a> LineNodeBuilder<'a> {
    pub fn style(mut self, style: Style) -> LineNodeBuilder<'a> {
        self.style = style;
        self
    }

    pub fn close_style(mut self) -> LineNodeBuilder<'a> {
        self.style = Style::empty();
        self
    }

    pub fn attrs(mut self, attributes: Attributes) -> LineNodeBuilder<'a> {
        self.style = self.style.add_attributes(attributes);
        self
    }

    pub fn close_attrs(mut self, attributes: Attributes) -> LineNodeBuilder<'a> {
        self.style = self.style.remove_attributes(attributes);
        self
    }

    pub fn close_fg(mut self) -> LineNodeBuilder<'a> {
        self.style = self.style.clear_fg();

        self
    }

    pub fn close_bg(mut self) -> LineNodeBuilder<'a> {
        self.style = self.style.clear_bg();
        self
    }

    pub fn add(mut self, item: impl Display + 'a) -> LineNodeBuilder<'a> {
        let string = format!("{}", item);

        if string.len() > 0 {
            let text = TextNode {
                data: FormattedText::String(Cow::Owned(string)),
                style: self.style.clone(),
            };

            self.parts.push(text);
        }

        self
    }

    pub fn add_option<Item: Display + 'a>(
        self,
        item: &'a Option<Item>,
        cb: impl FnOnce(LineNodeBuilder<'a>, &'a Item) -> LineNodeBuilder<'a>,
    ) -> LineNodeBuilder<'a> {
        if let Some(item) = item {
            cb(self, item)
        } else {
            self
        }
    }

    pub fn add_formatted(mut self, item: Arguments<'a>) -> LineNodeBuilder<'a> {
        let text = TextNode {
            data: FormattedText::Args(item),
            style: self.style.clone(),
        };

        self.parts.push(text);
        self
    }

    pub fn build(self) -> LineNode<'a> {
        LineNode { parts: self.parts }
    }
}

pub struct ComponentNode<'a> {
    pub(crate) lines: Vec<LineNode<'a>>,
}

impl<'a> ComponentNode<'a> {
    pub fn build() -> ComponentNodeBuilder<'a> {
        ComponentNodeBuilder::new()
    }
}

pub struct ComponentNodeBuilder<'a> {
    lines: Vec<LineNode<'a>>,
}

impl<'a> ComponentNodeBuilder<'a> {
    pub fn new() -> ComponentNodeBuilder<'a> {
        ComponentNodeBuilder { lines: vec![] }
    }

    pub fn line(mut self, line: LineNodeBuilder<'a>) -> ComponentNodeBuilder<'a> {
        self.lines.push(line.build());
        self
    }

    pub fn push(mut self, line: LineNodeBuilder<'a>) -> ComponentNodeBuilder<'a> {
        self.lines.push(line.build());
        self
    }

    pub fn build(self) -> ComponentNode<'a> {
        ComponentNode { lines: self.lines }
    }
}

pub struct DocumentNode<'a> {
    components: Vec<ComponentNode<'a>>,
}

impl<'a> DocumentNode<'a> {
    pub fn build() -> DocumentNodeBuilder<'a> {
        DocumentNodeBuilder { components: vec![] }
    }

    pub fn write<W: WriteColor>(&self, writer: &mut W) -> io::Result<()> {
        for component in &self.components {
            for line in &component.lines {
                writer.reset()?;

                for part in &line.parts {
                    writer.set_color(&part.style.to_color_spec())?;
                    write!(writer, "{}", part.data)?;
                }

                writeln!(writer, "")?;
            }
        }

        Ok(())
    }
}

pub struct DocumentNodeBuilder<'a> {
    components: Vec<ComponentNode<'a>>,
}

impl<'a> DocumentNodeBuilder<'a> {
    pub fn add(mut self, component: ComponentNode<'a>) -> DocumentNodeBuilder<'a> {
        self.components.push(component);
        self
    }

    pub fn build(self) -> DocumentNode<'a> {
        DocumentNode {
            components: self.components,
        }
    }
}
