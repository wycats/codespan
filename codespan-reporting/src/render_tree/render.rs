use super::{Document, Line, Node, Section};

/// The Render trait defines a type that can be added to a Document.
/// It is defined for `Node`, `String`, `&str`, and `Document`.alloc
///
/// It is also defined for `Option<T>` where `T` is `Render`, as well
/// as `&T` where `T` is both `Render` and `Clone`.
///
/// Generally speaking, if you need to make a type `Render`, and it's
/// not one of your types, you can ergonomically make a newtype wrapper
/// for it.
///
/// For example, if you want to render `std::time::Duration`:
///
/// ```
/// #[macro_use]
/// extern crate codespan_reporting;
/// extern crate termcolor;
/// use codespan_reporting::{Render, Document, RenderComponent};
/// use std::time::Duration;
/// use termcolor::StandardStream;
///
/// struct RenderDuration(Duration);
///
/// impl Render for RenderDuration {
///     fn render(self, into: Document) -> Document {
///         into.add(format!("{} seconds and {} nanos", self.0.as_secs(), self.0.subsec_nanos()))
///     }
/// }
///
/// struct MessageContents {
///     code: usize,
///     header: String,
///     body: String,
///     duration: Duration,
/// }
///
/// struct Message;
///
/// impl<'args> RenderComponent<'args> for Message {
///     type Args = MessageContents;
///
///     fn render(&self, args: MessageContents) -> Document {
///         tree! {
///             <line {
///                 {args.code} ":" {args.header} "for" {RenderDuration(args.duration)}
///             }>
///
///             <line {
///                 {args.body}
///             }>
///         }
///     }
/// }
///
/// fn main() -> std::io::Result<()> {
///     let message = MessageContents {
///         code: 200,
///         header: "Hello world".to_string(),
///         body: "This is the body of the message".to_string(),
///         duration: Duration::new(100, 1_000_000)
///     };
///
///     let document = tree! { <Message {message}> };
///
///     document.write()
/// }
/// ```
pub trait Render: Sized {
    /// Produce a new Document with `self` added to the `into` Document.
    fn render(self, into: Document) -> Document;

    fn into_fragment(self) -> Document {
        self.render(Document::empty())
    }

    fn add<Right: Render>(self, other: Right) -> Combine<Self, Right> {
        Combine {
            left: self,
            right: other,
        }
    }
}

pub struct Combine<Left: Render, Right: Render> {
    pub(crate) left: Left,
    pub(crate) right: Right,
}

impl<Left: Render, Right: Render> Render for Combine<Left, Right> {
    fn render(self, into: Document) -> Document {
        into.add(self.left).add(self.right)
    }
}

/// A node is rendered by adding itself to the document
impl Render for Node {
    fn render(self, document: Document) -> Document {
        document.add_node(self)
    }
}

// /// A String is rendered by turning itself into a text node and adding the
// /// text node into the document.
// impl Render for String {
//     fn render(self, document: Document) -> Document {
//         document.add_node(Node::Text(self))
//     }
// }

// /// A &str is rendered by turning itself into a String and rendering the
// /// String.
// impl<'a> Render for &'a str {
//     fn render(self, document: Document) -> Document {
//         self.to_string().render(document)
//     }
// }

/// A Document is rendered by extending its nodes onto the original
/// document.
impl Render for Document {
    fn render(self, into: Document) -> Document {
        into.extend(self)
    }
}

impl<Fragment: Render> Render for Section<Fragment> {
    fn render(self, into: Document) -> Document {
        into.add(Node::OpenSection(self.name))
            .add(self.fragment)
            .add(Node::CloseSection)
    }
}

impl<Fragment: Render> Render for Line<Fragment> {
    fn render(self, into: Document) -> Document {
        into.add(self.fragment).add(Node::Newline)
    }
}

// /// An Option<impl Render> is rendered by doing nothing if None or
// /// rendering the inner value if Some.
// impl<T> Render for Option<T>
// where
//     T: Render,
// {
//     fn render(self, document: Document) -> Document {
//         match self {
//             None => document,
//             Some(item) => item.render(document),
//         }
//     }
// }

#[allow(non_snake_case)]
pub fn IfSome(option: &Option<impl Render + Clone>) -> impl Render {
    // TODO: TAKE A CLOSURE LIKE EACH

    match option {
        None => Document::empty(),
        Some(inner) => inner.clone().render(Document::empty()),
    }
}

// /// An `&impl Render + Clone` is rendered by cloning the value and
// /// rendering it.
// impl<'a, T> Render for &'a T
// where
//     T: Render + Clone,
// {
//     fn render(self, document: Document) -> Document {
//         self.clone().render(document)
//     }
// }

impl<T: ::std::fmt::Display> Render for T {
    fn render(self, document: Document) -> Document {
        document.add(Node::Text(self.to_string()))
    }
}
