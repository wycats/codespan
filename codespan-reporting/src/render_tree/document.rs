use render_tree::{Display, Render};
use std::io;
use std::ops::Add;
use style::WriteStyle;
use termcolor::{ColorChoice, StandardStream, WriteColor};
use Stylesheet;

#[derive(Debug, Clone)]
pub enum Node {
    Text(String),
    OpenSection(&'static str),
    CloseSection,
    Newline,
}

#[derive(Debug, Clone)]
pub struct Document {
    // Make the inner tree optional so it's free to create empty documents
    tree: Option<Vec<Node>>,
}

impl Document {
    pub fn empty() -> Document {
        Document { tree: None }
    }

    pub fn into_tree(self) -> Option<Vec<Node>> {
        self.tree
    }

    pub fn tree(&self) -> Option<&[Node]> {
        match &self.tree {
            None => None,
            Some(vec) => Some(&vec[..]),
        }
    }

    fn initialize_tree(&mut self) -> &mut Vec<Node> {
        if self.tree.is_none() {
            self.tree = Some(vec![]);
        }

        match &mut self.tree {
            Some(value) => value,
            None => unreachable!(),
        }
    }

    pub fn add(self, renderable: impl Render) -> Document {
        self.extend(renderable.into_fragment())
    }

    pub fn add_node(mut self, node: Node) -> Document {
        self.initialize_tree().push(node);
        self
    }

    pub fn extend_nodes(mut self, other: Vec<Node>) -> Document {
        if other.len() > 0 {
            let tree = self.initialize_tree();

            for item in other {
                tree.push(item)
            }
        }

        self
    }

    pub fn extend(self, fragment: Document) -> Document {
        match (&self.tree, fragment.tree) {
            (Some(_), Some(other)) => self.extend_nodes(other),
            (Some(_), None) => self,
            (None, Some(nodes)) => Document { tree: Some(nodes) },
            (None, None) => Document::empty(),
        }
    }

    pub fn write(self) -> io::Result<()> {
        let mut writer = StandardStream::stdout(ColorChoice::Always);

        self.write_with(&mut writer, &Stylesheet::new())
    }

    pub fn write_with(
        self,
        writer: &mut impl WriteColor,
        stylesheet: &Stylesheet,
    ) -> io::Result<()> {
        let mut nesting = vec![];

        writer.reset()?;

        let tree = match self.tree {
            None => return Ok(()),
            Some(nodes) => nodes,
        };

        for item in tree {
            match item {
                Node::Text(string) => {
                    if string.len() != 0 {
                        let style = stylesheet.get(&nesting);

                        match style {
                            None => writer.reset()?,
                            Some(style) => writer.set_style(&style)?,
                        }

                        write!(writer, "{}", string)?;
                    }
                },
                Node::OpenSection(section) => nesting.push(section),
                Node::CloseSection => {
                    nesting.pop().expect("unbalanced push/pop");
                },
                Node::Newline => {
                    writer.reset()?;
                    write!(writer, "\n")?;
                },
            }
        }

        Ok(())
    }
}

impl Add<Document> for String {
    type Output = Document;

    fn add(self, other: Document) -> Document {
        let fragment = Display(self);

        match other.into_tree() {
            None => fragment,
            Some(tree) => fragment.extend_nodes(tree),
        }
    }
}

impl<T: Render> Add<T> for Document {
    type Output = Document;

    fn add(self, other: T) -> Document {
        other.render(self)
    }
}
