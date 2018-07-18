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

pub(crate) struct Section<R: Render> {
    pub(crate) name: &'static str,
    pub(crate) fragment: R,
}

/// A section that can be appended into a document. Sections are invisible, but
/// can be targeted in stylesheets with selectors using their name.
#[allow(non_snake_case)]
pub fn Section(name: &'static str, fragment: impl Render) -> impl Render {
    Section { name, fragment }
}

pub trait IterBlockHelper {
    type Args;
    type Item;

    fn args(Self::Args) -> Self;

    fn render(
        self,
        callback: impl Fn(Self::Item, Document) -> Document,
        document: Document,
    ) -> Document;
}

pub trait SimpleBlockHelper {
    fn render(self, callback: impl FnOnce(Document) -> Document, document: Document) -> Document;
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
///     <Each {items} |item| {
///         <line {
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

// #[allow(non_snake_case)]
// pub fn Each<'item, Items, T, F>(items: Items, callback: F) -> BlockComponent<Items, T, F>
// where
//     Items: IntoIterator<Item = &'item T> + 'item,
//     T: 'item,
//     F: Fn(&T, Document) -> Document,
// {
//     BlockComponent(EachComponent, items, callback)
// }
pub struct Each<U, Iterator: IntoIterator<Item = U>> {
    iterator: Iterator,
}

impl<'item, U, Iterator> IterBlockHelper for Each<U, Iterator>
where
    Iterator: IntoIterator<Item = U>,
{
    type Args = Iterator;
    type Item = U;

    fn args(iterator: Iterator) -> Each<U, Iterator> {
        Each { iterator }
    }

    fn render(
        self,
        callback: impl Fn(Self::Item, Document) -> Document,
        mut into: Document,
    ) -> Document {
        for item in self.iterator {
            into = callback(item, into);
        }

        into
    }
}

#[allow(non_snake_case)]
pub fn Each<U>(
    items: impl IntoIterator<Item = U>,
    callback: impl Fn(U, Document) -> Document,
) -> impl Render {
    IterBlockComponent(Each::args(items), callback)
}

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
///     &items,
///     ", ",
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

#[allow(non_snake_case)]
pub fn Join<U, F, Iterator>(iterator: Iterator, joiner: &'static str, callback: F) -> impl Render
where
    F: Fn(U, Document) -> Document,
    Iterator: IntoIterator<Item = U>,
{
    IterBlockComponent(Join { iterator, joiner }, callback)
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
pub struct Line;

impl SimpleBlockHelper for Line {
    fn render(self, callback: impl FnOnce(Document) -> Document, into: Document) -> Document {
        callback(into).add_node(Node::Newline)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_each() -> ::std::io::Result<()> {
        use render_tree::{Document, Each, Line, Render};
        struct Point(i32, i32);

        let items = &vec![Point(10, 20), Point(5, 10), Point(6, 42)][..];

        let document = tree! {
            <Each {items} |item| {
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
        use render_tree::{Document, Join, Render};
        struct Point(i32, i32);

        let items = &vec![Point(10, 20), Point(5, 10), Point(6, 42)][..];

        let document = tree! {
            <Join iterator={items} joiner={"\n"} |item| {
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
