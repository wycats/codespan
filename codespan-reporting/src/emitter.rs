use codespan::CodeMap;
use components;
use diagnostic::Diagnostic;
use log;
use render_tree::{Component, Render};
use std::{fmt, io};
use termcolor::WriteColor;
use Stylesheet;

pub fn emit<'doc, W>(
    writer: W,
    codemap: &'doc CodeMap,
    diagnostic: &'doc Diagnostic,
) -> io::Result<()>
where
    W: WriteColor,
{
    DiagnosticWriter { writer }.emit(DiagnosticData {
        codemap,
        diagnostic,
    })
}

struct DiagnosticWriter<W> {
    writer: W,
}

impl<W> DiagnosticWriter<W>
where
    W: WriteColor,
{
    fn emit<'doc>(mut self, data: DiagnosticData<'doc>) -> io::Result<()> {
        let document = Component(components::Diagnostic, data).into_fragment();

        let styles = Stylesheet::new()
            .add("** header **", "weight: bold")
            .add("bug ** primary", "fg: red")
            .add("error ** primary", "fg: red")
            .add("warning ** primary", "fg: yellow")
            .add("note ** primary", "fg: green")
            .add("help ** primary", "fg: cyan")
            .add("** secondary", "fg: blue")
            .add("** gutter", "fg: blue");

        if log_enabled!(log::Level::Debug) {
            document.debug_write(&mut self.writer, &styles)?;
        }

        document.write_with(&mut self.writer, &styles)?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct DiagnosticData<'doc> {
    pub(crate) codemap: &'doc CodeMap,
    pub(crate) diagnostic: &'doc Diagnostic,
}

pub fn format(f: impl Fn(&mut fmt::Formatter) -> fmt::Result) -> impl fmt::Display {
    struct Display<F>(F);

    impl<F> fmt::Display for Display<F>
    where
        F: Fn(&mut fmt::Formatter) -> fmt::Result,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            (self.0)(f)
        }
    }
    Display(f)
}

#[cfg(test)]
mod default_emit_smoke_tests {
    use super::*;
    use codespan::*;
    use regex;
    use std::io::{self, Write};
    use termcolor::{Buffer, ColorSpec};
    use unindent::unindent;
    use *;

    fn emit_with_writer<W: WriteColor>(mut writer: W) -> W {
        let mut code_map = CodeMap::new();

        let source = unindent(
            r##"
                (define test 123)
                (+ test "")
                ()
            "##,
        );

        let file_map = code_map.add_filemap("test".into(), source.to_string());

        let str_start = file_map.byte_index(1.into(), 8.into()).unwrap();
        let error = Diagnostic::new(Severity::Error, "Unexpected type in `+` application")
            .with_label(
                Label::new_primary(Span::from_offset(str_start, 2.into()))
                    .with_message("Expected integer but got string"),
            )
            .with_label(
                Label::new_secondary(Span::from_offset(str_start, 2.into()))
                    .with_message("Expected integer but got string"),
            )
            .with_code("E0001");

        let line_start = file_map.byte_index(1.into(), 0.into()).unwrap();
        let warning = Diagnostic::new(
            Severity::Warning,
            "`+` function has no effect unless its result is used",
        ).with_label(Label::new_primary(Span::from_offset(line_start, 11.into())));

        let diagnostics = [error, warning];

        for diagnostic in &diagnostics {
            emit(&mut writer, &code_map, &diagnostic).unwrap();
        }

        writer
    }

    #[test]
    fn test_no_color() {
        assert_eq!(
            String::from_utf8_lossy(&emit_with_writer(Buffer::no_color()).into_inner()),
            unindent(&format!(
                r##"
                    error[E0001]: Unexpected type in `+` application
                    - <test>:2:9
                    2 | (+ test "")
                      |         ^^ Expected integer but got string
                    - <test>:2:9
                    2 | (+ test "")
                      |         -- Expected integer but got string
                    warning: `+` function has no effect unless its result is used
                    - <test>:2:1
                    2 | (+ test "")
                      | ^^^^^^^^^^^
                "##,
            )),
        );
    }

    /// A facility for creating visually inspectable representations of colored output
    /// so they can be easily tested.
    ///
    /// A new color is represented as `{style}` and a reset is represented by `{/}`.
    ///
    /// Attributes are printed in this order:
    ///
    /// - Foreground color as `fg:Color`
    /// - Background color as `bg:Color`
    /// - Bold as `bold`
    /// - Underline as `underline`
    /// - Intense as `bright`
    ///
    /// For example, the style "intense, bold red foreground" would be printed as:
    ///
    /// ```
    /// {fg:Red bold intense}
    /// ```
    ///
    /// Since this implementation attempts to make it possible to faithfully
    /// understand what real WriteColor implementations would do, it tries
    /// to approximate the contract in the WriteColor trait: "Subsequent
    /// writes to this write will use these settings until either reset is
    /// called or new color settings are set.")
    ///
    /// - If set_color is called with a style, `{...}` is emitted containing the
    ///   color attributes.
    /// - If set_color is called with no style, `{/}` is emitted
    /// - If reset is called, `{/}` is emitted
    struct ColorAccumulator {
        buf: Vec<u8>,
        color: ColorSpec,
    }

    impl ColorAccumulator {
        fn new() -> ColorAccumulator {
            ColorAccumulator {
                buf: Vec::new(),
                color: ColorSpec::new(),
            }
        }

        fn to_string(self) -> String {
            String::from_utf8(self.buf).unwrap()
        }
    }

    impl io::Write for ColorAccumulator {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buf.extend(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl WriteColor for ColorAccumulator {
        fn supports_color(&self) -> bool {
            true
        }

        fn set_color(&mut self, spec: &termcolor::ColorSpec) -> io::Result<()> {
            #![allow(unused_assignments)]

            if self.color == *spec {
                return Ok(());
            } else {
                trace!(
                    "eq={:?} spec={:?} current={:?}",
                    self.color == *spec,
                    spec,
                    self.color
                );

                self.color = spec.clone();
            }

            if spec.is_none() {
                write!(self, "{{/}}")?;
                return Ok(());
            } else {
                write!(self, "{{")?;
            }

            let mut first = true;

            fn write_first(first: bool, write: &mut ColorAccumulator) -> io::Result<bool> {
                if !first {
                    write!(write, " ")?;
                }

                Ok(false)
            };

            if let Some(fg) = spec.fg() {
                first = write_first(first, self)?;
                write!(self, "fg:{:?}", fg)?;
            }

            if let Some(bg) = spec.bg() {
                first = write_first(first, self)?;
                write!(self, "bg:{:?}", bg)?;
            }

            if spec.bold() {
                first = write_first(first, self)?;
                write!(self, "bold")?;
            }

            if spec.underline() {
                first = write_first(first, self)?;
                write!(self, "underline")?;
            }

            if spec.intense() {
                first = write_first(first, self)?;
                write!(self, "bright")?;
            }

            write!(self, "}}")?;

            Ok(())
        }

        fn reset(&mut self) -> io::Result<()> {
            let color = self.color.clone();

            if color != ColorSpec::new() {
                write!(self, "{{/}}")?;
                self.color = ColorSpec::new();
            }

            Ok(())
        }
    }

    #[test]
    fn test_color() {
        assert_eq!(
            emit_with_writer(ColorAccumulator::new()).to_string(),

            normalize(
                r#"
                   {fg:Red bold bright} $$error[E0001]{bold bright}: Unexpected type in `+` application{/}
                                        $$- <test>:2:9
                              {fg:Cyan} $$2 | {/}(+ test {fg:Red}""{/})
                              {fg:Cyan} $$  | {/}        {fg:Red}^^ Expected integer but got string{/}
                                        $$- <test>:2:9
                              {fg:Cyan} $$2 | {/}(+ test {fg:Cyan}""{/})
                              {fg:Cyan} $$  | {/}        {fg:Cyan}-- Expected integer but got string{/}
                {fg:Yellow bold bright} $$warning{bold bright}: `+` function has no effect unless its result is used{/}
                                        $$- <test>:2:1
                              {fg:Cyan} $$2 | {fg:Yellow}(+ test ""){/}
                              {fg:Cyan} $$  | {fg:Yellow}^^^^^^^^^^^{/}
            "#
            )
        );
    }

    fn split_line<'a>(line: &'a str, by: &str) -> (&'a str, &'a str) {
        let mut splitter = line.splitn(2, by);
        let first = splitter.next().unwrap_or("");
        let second = splitter.next().unwrap_or("");
        (first, second)
    }

    fn normalize(s: impl AsRef<str>) -> String {
        let s = s.as_ref();
        let s = unindent(s);

        let regex = regex::Regex::new(r"\{-*\}").unwrap();

        s.lines()
            .map(|line| {
                let (style, line) = split_line(line, " $$");
                let line = regex.replace_all(&line, "").to_string();
                format!("{style}{line}\n", style = style.trim(), line = line)
            })
            .collect()
    }
}
