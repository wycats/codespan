use render_tree::{Document, Node, Render};
use std::fmt;

#[allow(non_snake_case)]
pub fn Display(string: impl fmt::Display) -> Document {
    Document::empty() + Node::Text(format!("{}", string))
}

pub fn pad(item: impl fmt::Display, size: usize) -> Document {
    Display(PadItem(item, size))
}

pub struct PadItem<T>(pub T, pub usize);

impl<T: fmt::Display> fmt::Display for PadItem<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..(self.1) {
            self.0.fmt(f)?;
        }
        Ok(())
    }
}

pub struct Section {
    name: &'static str,
    fragment: Document,
}

impl Section {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn into_fragment(self) -> Document {
        self.fragment
    }
}

#[allow(non_snake_case)]
pub fn Section(name: &'static str, fragment: impl Render) -> Document {
    Document::empty() + Section {
        name,
        fragment: fragment.render(Document::empty()),
    }
}

#[allow(non_snake_case)]
pub fn Each<'item, T: 'item>(
    items: impl IntoIterator<Item = &'item T>,
    callback: impl Fn(&T) -> Document,
) -> Document {
    let mut document = Document::empty();

    for item in items {
        document = document.extend(callback(&item));
    }

    document
}

#[allow(non_snake_case)]
pub fn Line(document: Document) -> Document {
    document + Node::Newline
}
