use render_tree::{Document, Render};

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
/// use codespan_reporting::{Document, RenderComponent};
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
///                 {args.code} ":" {args.header}
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

/// A Component is an instance of RenderComponent and its args. Component
/// implements Render, so it can be added to a document during the render
/// process.
pub struct Component<'args, C: RenderComponent<'args>> {
    component: C,
    args: C::Args,
}

impl<'args, C: RenderComponent<'args>> Component<'args, C> {
    pub(crate) fn call(self) -> Document {
        self.component.render(self.args)
    }
}

#[allow(non_snake_case)]
pub fn Component<'args, C>(component: C, args: C::Args) -> Component<'args, C>
where
    C: RenderComponent<'args> + 'args,
{
    Component { component, args }
}

/// A Component is rendered by calling the component's render with
/// its args.
impl<'args, C> Render for Component<'args, C>
where
    C: RenderComponent<'args>,
{
    fn render(self, into: Document) -> Document {
        into.add(self.call())
    }
}
