//! This module defines a tree structure that you can use to build documents
//! to render with color.
//!
//! There are three kinds of nodes in the Render Tree:
//!
//! - Lines, which represent content followed by a newline
//! - Sections, which represent named content that can be targeted by
//!   style rules.
//! - Components, which are parameterized chunks of the Render Tree
//!
//! The easiest way to build a render tree is using the `tree!` macro:
//!
//! ```
//! #[macro_use]
//! extern crate codespan_reporting;
//! extern crate termcolor;
//! use codespan_reporting::{Document, RenderComponent};
//! use termcolor::StandardStream;
//!
//! fn main() -> std::io::Result<()> {
//!     let world = "world";
//!
//!     let document = tree! {
//!         <line {
//!             "Hello" {world}
//!         }>
//!
//!         <line {
//!             "Goodbye" {world}
//!         }>
//!     };
//!
//!     document.write()
//! }
//! ```
//!
//! This will print out:
//!
//! ```text
//! Hello world
//! Goodbye world
//!
//! ```
//!
//! You can use sections and stylesheets to colorize the output:
//!
//! ```
//! #[macro_use]
//! extern crate codespan_reporting;
//! extern crate termcolor;
//! use codespan_reporting::{Document, RenderComponent, Stylesheet};
//! use termcolor::StandardStream;
//!
//! fn main() -> std::io::Result<()> {
//!     let world = "world";
//!
//!     let document = tree! {
//!         <line {
//!             <section name="hello" { "Hello" }>
//!             {world}
//!         }>
//!
//!         <line {
//!             <section name="goodbye" { "Goodbye"}>
//!             {world}
//!         }>
//!     };
//!
//!     let stylesheet = Stylesheet::new()
//!         .add("hello", "fg: blue")
//!         .add("goodbye", "fg: red");
//!
//!     document.write_styled(&stylesheet)
//! }
//! ```
//!
//! This will print out:
//!
//! ```text
//! Hello world
//! Goodbye world
//!
//! ```
//!
//! with the word "Hello" colored blue and "Goodbye" colored red.
//!
//! You can nest sections, which can be targeted by style paths:
//!
//!
//! ```
//! #[macro_use]
//! extern crate codespan_reporting;
//! extern crate termcolor;
//! use codespan_reporting::{Document, RenderComponent, Stylesheet};
//! use termcolor::StandardStream;
//!
//! fn main() -> std::io::Result<()> {
//!     let world = "world";
//!
//!     let document = tree! {
//!         <line {
//!             <section name="hello-world" {
//!                 <section name="greeting" { "Hello" }>
//!                 {world}
//!             }>
//!         }>
//!
//!         <line {
//!             "Some content in the middle here"
//!         }>
//!
//!         <line {
//!             <section name="goodbye-world" {
//!                 <section name="greeting" { "Goodbye" }>
//!                 {world}
//!             }>
//!         }>
//!     };
//!
//!     let stylesheet = Stylesheet::new()
//!         .add("** greeting", "weight: bold")
//!         .add("hello-world greeting", "fg: red")
//!         .add("goodbye-world greeting", "fg: blue");
//!
//!     document.write_styled(&stylesheet)
//! }
//! ```
//!
//! This will print out:
//!
//! ```text
//! Hello world
//! Some content in the middle here
//! Goodbye world
//!
//! ```
//!
//! with the "Hello world" and "Goodbye world" bolded, the word "Hello" colored
//! red (and bolded, of course), and the word "Goodbye" colored red (and
//! bolded).
//!
//! Globs (`**`) in a rule are superseded by stars (`*`), which are supersede by
//! literal names. Globs match zero or more section names and stars match exactly
//! one section name.
//!
//! # Using without the `tree!` macro
//!
//! It's also easy to build a Render Tree without the macro. Repeating the previous
//! example without the macro and without the string DSL for the stylesheet:
//!
//! ```
//! #[macro_use]
//! extern crate codespan_reporting;
//! extern crate termcolor;
//! use codespan_reporting::{
//!     Color,
//!     Document,
//!     Line,
//!     Render,
//!     RenderComponent,
//!     Section,
//!     Selector,
//!     Segment,
//!     Style,
//!     Stylesheet
//! };
//! use termcolor::StandardStream;
//!
//! fn main() -> std::io::Result<()> {
//!     let world = "world";
//!     
//!     let document = Document::with(
//!         Line(
//!             Section("hello-world",
//!                 Section("greeting", "Hello").add(world)
//!             )
//!         )
//!     ).add(
//!         Line(
//!             "Some content in the middle here"
//!         )
//!     ).add(
//!         Line(
//!             Section("goodbye-world",
//!                 Section("greeting", "Goodbye").add(world)
//!             )
//!         )
//!     );
//!
//!     let stylesheet = Stylesheet::new()
//!         .add(Selector::glob().add("greeting"), Style::new().bold())
//!         .add(Selector::name("hello-world").add("greeting"), Style::new().fg(Color::Red))
//!         .add(Selector::name("goodbye-world").add("greeting"), Style::new().fg(Color::Blue));
//!
//!     document.write_styled(&stylesheet)
//! }
//! ```

#[macro_use]
pub mod macros;
mod component;
mod debug;
pub mod document;
mod helpers;
mod render;
pub mod stylesheet;
pub(crate) mod utils;

pub use self::component::*;
pub use self::document::*;
pub use self::helpers::*;
pub use self::render::{Combine, IfSome, Render};
pub use self::stylesheet::{Segment, Selector, Stylesheet};
