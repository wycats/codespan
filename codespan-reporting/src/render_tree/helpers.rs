use render_tree::component::{IterBlockHelper, OnceBlock, OnceBlockComponent, OnceBlockHelper};
use render_tree::{Document, IterBlockComponent, Node, Render};
use std::fmt;

/// Creates a `Render` that, when appended into a [`Document`], repeats
/// a given string a specified number of times.
pub fn repeat(item: impl fmt::Display, size: usize) -> impl Render {
    PadItem(item, size)
}

pub(crate) struct PadItem<T>(pub T, pub usize);

impl<T: fmt::Display> fmt::Display for PadItem<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..(self.1) {
            self.0.fmt(f)?;
        }
        Ok(())
    }
}

/// A list of items that can be appended into a [`Document`]. For each item in
/// `items`, the callback is invoked, and its return value is appended to
/// the document.
///
/// # Example
///
/// ```
/// # use codespan_reporting::{Document, Each, Line, Render, RenderComponent};
/// #
/// # fn main() -> Result<(), ::std::io::Error> {
/// struct Point(i32, i32);
///
/// let items = vec![Point(10, 20), Point(5, 10), Point(6, 42)];
///
/// let document = Document::with(Each(
///     &items,
///     |item, doc| doc.add(Line("Point(".add(item.0).add(",").add(item.1).add(")")))
/// ));
///
/// assert_eq!(document.to_string()?, "Point(10,20)\nPoint(5,10)\nPoint(6,42)\n");
/// #
/// # Ok(())
/// # }
/// ```
///
/// And with the [`tree!`] macro:
///
/// ```
/// # #[macro_use]
/// # extern crate codespan_reporting;
/// # use codespan_reporting::{Document, Each, Line, Render, RenderComponent};
/// #
/// # fn main() -> Result<(), ::std::io::Error> {
/// struct Point(i32, i32);
///
/// let items = vec![Point(10, 20), Point(5, 10), Point(6, 42)];
///
/// let document = tree! {
///     <Each args={items} as |item| {
///         <Line as {
///             "Point(" {item.0} "," {item.1} ")"
///         }>
///     }>
/// };
///
/// assert_eq!(document.to_string()?, "Point(10,20)\nPoint(5,10)\nPoint(6,42)\n");
/// #
/// # Ok(())
/// # }
/// ```

pub struct Each<U, Iterator: IntoIterator<Item = U>> {
    items: Iterator,
}

impl<'item, U, Iterator> IterBlockHelper for Each<U, Iterator>
where
    Iterator: IntoIterator<Item = U>,
{
    type Args = Iterator;
    type Item = U;

    fn args(items: Iterator) -> Each<U, Iterator> {
        Each { items }
    }

    fn render(
        self,
        callback: impl Fn(Self::Item, Document) -> Document,
        mut into: Document,
    ) -> Document {
        for item in self.items {
            into = callback(item, into);
        }

        into
    }
}

impl<U, I: IntoIterator<Item = U>> From<I> for Each<U, I> {
    fn from(from: I) -> Each<U, I> {
        Each { items: from }
    }
}

#[allow(non_snake_case)]
pub fn Each<U, I: IntoIterator<Item = U>>(
    items: impl Into<Each<U, I>>,
    callback: impl Fn(U, Document) -> Document,
) -> impl Render {
    IterBlockComponent(items.into(), callback)
}

///

/// A section that can be appended into a document. Sections are invisible, but
/// can be targeted in stylesheets with selectors using their name.
pub struct Section {
    pub name: &'static str,
}

impl OnceBlockHelper for Section {
    type Args = Section;
    type Item = ();

    fn args(args: Section) -> Section {
        args
    }

    fn render(
        self,
        callback: impl FnOnce((), Document) -> Document,
        mut into: Document,
    ) -> Document {
        into = into.add_node(Node::OpenSection(self.name));
        into = callback((), into);
        into.add_node(Node::CloseSection)
    }
}

impl From<&'static str> for Section {
    fn from(from: &'static str) -> Section {
        Section { name: from }
    }
}

#[allow(non_snake_case)]
pub fn Section(
    section: impl Into<Section>,
    callback: impl FnOnce(Document) -> Document,
) -> impl Render {
    OnceBlockComponent(section.into(), |_, document| callback(document))
}

///

/// Equivalent to [`Each`], but inserts a joiner between two adjacent elements.
///
/// # Example
///
/// ```
/// # use codespan_reporting::{Document, Join, Line, Render, RenderComponent};
/// #
/// # fn main() -> Result<(), ::std::io::Error> {
/// struct Point(i32, i32);
///
/// let items = vec![Point(10, 20), Point(5, 10), Point(6, 42)];
///
/// let document = Document::with(Join(
///     (&items, ", "),
///     |item, doc| doc.add("Point(").add(item.0).add(",").add(item.1).add(")")
/// ));
///
/// assert_eq!(document.to_string()?, "Point(10,20), Point(5,10), Point(6,42)");
///
/// # Ok(())
/// # }
/// ```
pub struct Join<U, Iterator: IntoIterator<Item = U>> {
    pub iterator: Iterator,
    pub joiner: &'static str,
}

impl<U, I: IntoIterator<Item = U>> From<(I, &'static str)> for Join<U, I> {
    fn from(from: (I, &'static str)) -> Join<U, I> {
        Join {
            iterator: from.0,
            joiner: from.1,
        }
    }
}

#[allow(non_snake_case)]
pub fn Join<U, F, Iterator>(join: impl Into<Join<U, Iterator>>, callback: F) -> impl Render
where
    F: Fn(U, Document) -> Document,
    Iterator: IntoIterator<Item = U>,
{
    IterBlockComponent(join.into(), callback)
}

impl<'item, U, Iterator> IterBlockHelper for Join<U, Iterator>
where
    Iterator: IntoIterator<Item = U>,
{
    type Args = Join<U, Iterator>;
    type Item = U;

    fn args(join: Join<U, Iterator>) -> Join<U, Iterator> {
        join
    }

    fn render(
        self,
        callback: impl Fn(Self::Item, Document) -> Document,
        mut into: Document,
    ) -> Document {
        let mut is_first = true;

        for item in self.iterator {
            if is_first {
                is_first = false;
            } else {
                into = into.add(self.joiner);
            }

            into = callback(item, into);
        }

        into
    }
}

/// Inserts a line into a [`Document`]. The contents are inserted first, followed
/// by a newline.
#[allow(non_snake_case)]
pub fn Line(item: impl Render) -> impl Render {
    OnceBlock(|document| item.render(document).add_node(Node::Newline))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_each() -> ::std::io::Result<()> {
        struct Point(i32, i32);

        let items = &vec![Point(10, 20), Point(5, 10), Point(6, 42)][..];

        let document = tree! {
            <Each args={items} as |item| {
                <Line as {
                    "Point(" {item.0} "," {item.1} ")"
                }>
            }>
        };

        assert_eq!(
            document.to_string()?,
            "Point(10,20)\nPoint(5,10)\nPoint(6,42)\n"
        );

        Ok(())
    }

    #[test]
    fn test_join() -> ::std::io::Result<()> {
        struct Point(i32, i32);

        let items = &vec![Point(10, 20), Point(5, 10), Point(6, 42)][..];

        let document = tree! {
            <Join iterator={items} joiner={"\n"} as |item| {
                "Point(" {item.0} "," {item.1} ")"
            }>
        };

        assert_eq!(
            document.to_string()?,
            "Point(10,20)\nPoint(5,10)\nPoint(6,42)"
        );

        Ok(())
    }
}
