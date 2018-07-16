use render_tree::{Document, Render};
use std::ops::Add;

/// This trait defines a renderable entity with arguments. Types that implement
/// `RenderComponent` can be packaged up together with their arguments in a
/// `Component`, and the `Component` is renderable.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate codespan_reporting;
/// extern crate termcolor;
/// use codespan_reporting::{Document, Display, RenderComponent};
/// use termcolor::StandardStream;
///
/// struct MessageContents {
///     code: usize,
///     header: String,
///     body: String,
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
///                 {Display(args.code)} ":" {args.header}
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
///         body: "This is the body of the message".to_string()
///     };
///
///     let document = tree! { <Message {message}> };
///
///     document.write()
/// }
/// ```
pub trait RenderComponent<'args> {
    type Args;

    fn render(&self, args: Self::Args) -> Document;
}

pub(crate) struct Component<'args, C: RenderComponent<'args>> {
    component: C,
    args: C::Args,
}

impl<'args, C: RenderComponent<'args>> Component<'args, C> {
    pub(crate) fn call(self) -> Document {
        self.component.render(self.args)
    }
}

#[allow(non_snake_case)]
pub fn Component<'args, C>(component: C, args: C::Args) -> Document
where
    C: RenderComponent<'args>,
{
    let document = Document::empty();
    document.add(Component { component, args })
}

impl<'args, C1: RenderComponent<'args>, C2: RenderComponent<'args>> Add<Component<'args, C2>>
    for Component<'args, C1>
{
    type Output = Document;

    fn add(self, other: Component<'args, C2>) -> Document {
        let mut fragment = Document::empty();
        fragment = fragment.add(self);
        fragment = fragment.add(other);
        fragment
    }
}

/// A Component is rendered by calling the component's render with
/// its args.
impl<'args, C> Render for Component<'args, C>
where
    C: RenderComponent<'args>,
{
    fn render(self, document: Document) -> Document {
        document + self.call()
    }
}
