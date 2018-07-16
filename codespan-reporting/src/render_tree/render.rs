use super::{Document, Node, Section};

pub trait Render: Sized {
    /// Produce a new Document with `self` added to the `into` Document.
    fn render(self, into: Document) -> Document;

    fn into_fragment(self) -> Document {
        self.render(Document::empty())
    }
}

/// A node is rendered by adding itself to the document
impl Render for Node {
    fn render(self, document: Document) -> Document {
        document.add_node(self)
    }
}

impl Render for fn(document: Document) -> Document {
    fn render(self, document: Document) -> Document {
        self(document)
    }
}

/// A String is rendered by turning itself into a text node and adding the
/// text node into the document.
impl Render for String {
    fn render(self, document: Document) -> Document {
        document.add_node(Node::Text(self))
    }
}

/// A &str is rendered by turning itself into a String and rendering the
/// String.
impl<'a> Render for &'a str {
    fn render(self, document: Document) -> Document {
        self.to_string().render(document)
    }
}

/// A Document is rendered by extending its nodes onto the original
/// document.
impl Render for Document {
    fn render(self, into: Document) -> Document {
        into.extend(self)
    }
}

/// An Option<impl Render> is rendered by doing nothing if None or
/// rendering the inner value if Some.
impl<T> Render for Option<T>
where
    T: Render,
{
    fn render(self, document: Document) -> Document {
        match self {
            None => document,
            Some(item) => item.render(document),
        }
    }
}

/// An `&impl Render + Clone` is rendered by cloning the value and
/// rendering it.
impl<'a, T> Render for &'a T
where
    T: Render + Clone,
{
    fn render(self, document: Document) -> Document {
        self.clone().render(document)
    }
}

/// A Section is rendered by adding an `OpenSection` node, extending
/// the document with the section's fragment, and adding a `CloseSection`
/// node.
impl Render for Section {
    fn render(self, mut document: Document) -> Document {
        document = document.add(Node::OpenSection(self.name()));
        document = document.extend(self.into_fragment());
        document = document.add(Node::CloseSection);
        document
    }
}
