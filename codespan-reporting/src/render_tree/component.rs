use render_tree::{Document, Render};

pub trait OnceBlockHelper {
    type Args;
    type Item;

    fn args(Self::Args) -> Self;

    fn render(
        self,
        callback: impl FnOnce(Self::Item, Document) -> Document,
        document: Document,
    ) -> Document;
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
/// use codespan_reporting::{Document, Line, Render, RenderComponent};
/// use termcolor::StandardStream;
///
/// struct MessageContents {
///     code: usize,
///     header: String,
///     body: String,
/// }
///
/// fn Message(args: MessageContents, into: Document) -> Document {
///     into.add(tree! {
///         <Line as {
///             {args.code} ":" {args.header}
///         }>
///
///         <Line as {
///             {args.body}
///         }>
///     })
/// }
///
/// fn main() -> std::io::Result<()> {
///     let message = MessageContents {
///         code: 200,
///         header: "Hello world".to_string(),
///         body: "This is the body of the message".to_string()
///     };
///
///     let document = tree! { <Message args={message}> };
///
///     document.write()
/// }
/// ```
pub trait RenderComponent<'args> {
    type Args;

    fn render(&self, args: Self::Args, into: Document) -> Document;
}

type ComponentFn<Args> = fn(Args, Document) -> Document;

/// A Component is an instance of RenderComponent and its args. Component
/// implements Render, so it can be added to a document during the render
/// process.
pub struct Component<Args> {
    component: ComponentFn<Args>,
    args: Args,
}

#[allow(non_snake_case)]
pub fn Component<Args>(component: ComponentFn<Args>, args: Args) -> Component<Args> {
    Component { component, args }
}

/// A Component is rendered by calling the component's render with
/// its args.
impl<Args> Render for Component<Args> {
    fn render(self, into: Document) -> Document {
        (self.component)(self.args, into)
    }
}

pub struct OnceBlockComponent<B: OnceBlockHelper, F: FnOnce(B::Item, Document) -> Document> {
    helper: B,
    callback: F,
}

impl<B, F> Render for OnceBlockComponent<B, F>
where
    B: OnceBlockHelper,
    F: FnOnce(B::Item, Document) -> Document,
{
    fn render(self, into: Document) -> Document {
        (self.helper).render(self.callback, into)
    }
}

#[allow(non_snake_case)]
pub fn OnceBlockComponent<B, F>(helper: B, callback: F) -> OnceBlockComponent<B, F>
where
    B: OnceBlockHelper,
    F: FnOnce(B::Item, Document) -> Document,
{
    OnceBlockComponent { helper, callback }
}

pub struct IterBlockComponent<B: IterBlockHelper, F: Fn(B::Item, Document) -> Document> {
    helper: B,
    callback: F,
}

impl<B, F> Render for IterBlockComponent<B, F>
where
    B: IterBlockHelper,
    F: Fn(B::Item, Document) -> Document,
{
    fn render(self, into: Document) -> Document {
        (self.helper).render(self.callback, into)
    }
}

#[allow(non_snake_case)]
pub fn IterBlockComponent<B, F>(helper: B, callback: F) -> IterBlockComponent<B, F>
where
    B: IterBlockHelper,
    F: Fn(B::Item, Document) -> Document,
{
    IterBlockComponent { helper, callback }
}

pub struct OnceBlock<F: FnOnce(Document) -> Document>(pub F);

impl<F> Render for OnceBlock<F>
where
    F: FnOnce(Document) -> Document,
{
    fn render(self, into: Document) -> Document {
        (self.0)(into)
    }
}
