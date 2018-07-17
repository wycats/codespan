use render_tree::{Document, Render};
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
///     |item| Line("Point(".add(item.0).add(",").add(item.1).add(")"))
/// ));
///
/// assert_eq!(document.to_string()?, "Point(10,20)\nPoint(5,10)\nPoint(6,42)\n");
/// #
/// # Ok(())
/// # }
/// ```
#[allow(non_snake_case)]
pub fn Each<'item, T: 'item, R: Render>(
    items: impl IntoIterator<Item = &'item T>,
    callback: impl Fn(&T) -> R,
) -> impl Render {
    let mut document = Document::empty();

    for item in items {
        document = document.add(callback(&item));
    }

    document
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
///     |item| "Point(".add(item.0).add(",").add(item.1).add(")")
/// ));
///
/// assert_eq!(document.to_string()?, "Point(10,20), Point(5,10), Point(6,42)");
///
/// # Ok(())
/// # }
/// ```
#[allow(non_snake_case)]
pub fn Join<'item, T: 'item, R: Render>(
    items: impl IntoIterator<Item = &'item T>,
    joiner: impl fmt::Display,
    callback: impl Fn(&T) -> R,
) -> impl Render {
    let mut document = Document::empty();
    let mut is_first = true;

    for item in items {
        if is_first {
            is_first = false;
        } else {
            document = document.add(&joiner);
        }

        document = document.add(callback(&item));
    }

    document
}

/// Inserts a line into a [`Document`]. The contents are inserted first, followed
/// by a newline.
pub(crate) struct Line<R: Render> {
    pub(crate) fragment: R,
}

#[allow(non_snake_case)]
pub fn Line(fragment: impl Render) -> impl Render {
    Line { fragment }
}
