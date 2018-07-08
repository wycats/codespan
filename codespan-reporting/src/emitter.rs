use codespan::{ByteSpan, CodeMap, ColumnIndex, FileMap, FileName, LineIndex};
use emit_config::{DefaultConfig, EmitConfig};
use std::{fmt, io};
use termcolor::WriteColor;
use DocumentNode;

use {Diagnostic, Label};

struct Pad<T>(T, usize);

impl<T: fmt::Display> fmt::Display for Pad<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..(self.1) {
            self.0.fmt(f)?;
        }
        Ok(())
    }
}

pub fn emit<'diagnostic, 'codemap: 'diagnostic, W>(
    writer: W,
    codemap: &'codemap CodeMap,
    diagnostic: &'diagnostic Diagnostic,
) -> io::Result<()>
where
    W: WriteColor,
{
    DiagnosticWriter { writer }.emit(DiagnosticData {
        codemap,
        diagnostic,
        config: DefaultConfig,
    })
}

pub(crate) mod components {
    use super::{Pad, SourceLine};
    use emit_config::EmitConfig;
    use render_tree::*;
    use {Diagnostic, Label, Severity};

    pub(crate) struct Header {
        severity: Severity,
        code: Option<String>,
        message: String,
    }

    impl Header {
        pub(crate) fn from_diagnostic(diagnostic: &Diagnostic) -> Header {
            Header {
                severity: diagnostic.severity,
                code: diagnostic.code.clone(),
                message: diagnostic.message.clone(),
            }
        }

        pub(crate) fn render<'codemap>(&self, config: &impl EmitConfig<'codemap>) -> ComponentNode {
            let highlight_style = config
                .highlight_style()
                .fg(config.severity_color(self.severity));

            ComponentNode::build()
                .line({
                    let mut line = LineNode::build()
                        .style(highlight_style)
                        .add(self.severity.to_string());

                    if let Some(code) = &self.code {
                        line = line.add("[").add(code).add("]");
                    }

                    line.close_fg().add(": ").add(&self.message)
                })
                .build()
        }
    }

    pub(crate) struct CodeLine<'diagnostic> {
        message: &'diagnostic Option<String>,
    }

    impl<'diagnostic> CodeLine<'diagnostic> {
        pub(crate) fn from_message(message: &'diagnostic Option<String>) -> CodeLine<'diagnostic> {
            CodeLine { message }
        }

        pub(crate) fn render<'codemap>(
            &self,
            _config: &impl EmitConfig<'codemap>,
        ) -> ComponentNode<'diagnostic> {
            ComponentNode::build()
                .line(
                    LineNode::build()
                        .add_option(self.message, |line, message| line.add("- ").add(message)),
                )
                .build()
        }
    }

    pub(crate) struct SourceCodeLocation<'codemap> {
        source_line: SourceLine<'codemap>,
    }

    impl<'codemap> SourceCodeLocation<'codemap> {
        pub(crate) fn from_source_line(
            source_line: SourceLine<'codemap>,
        ) -> SourceCodeLocation<'codemap> {
            SourceCodeLocation { source_line }
        }

        pub(crate) fn render(
            &self,
            _config: &impl EmitConfig<'codemap>,
        ) -> ComponentNode<'codemap> {
            ComponentNode::build()
                .line({
                    let (line, column) = self.source_line.location();
                    let filename = self.source_line.filename().to_string();

                    LineNode::build()
                        .add("- ")
                        .add(filename)
                        .add(":")
                        .add(line.number())
                        .add(":")
                        .add(column.number())
                })
                .build()
        }
    }

    pub(crate) struct SourceCodeLine<'codemap> {
        source_line: SourceLine<'codemap>,
        label: &'codemap Label,
        severity: Severity,
    }

    impl<'codemap> SourceCodeLine<'codemap> {
        pub(crate) fn from_message(
            source_line: SourceLine<'codemap>,
            label: &'codemap Label,
            severity: Severity,
        ) -> SourceCodeLine<'codemap> {
            SourceCodeLine {
                source_line,
                label,
                severity,
            }
        }

        pub(crate) fn render(&self, config: &impl EmitConfig<'codemap>) -> ComponentNode<'codemap> {
            ComponentNode::build()
                .line({
                    LineNode::build()
                        .style(config.line_location_style())
                        .add(self.source_line.location().0.number())
                        .add(" ")
                        .add(config.gutter_separator())
                        .add(" ")
                        .close_style()
                        .add(self.source_line.prefix())
                        .style(config.label_style(self.severity, self.label))
                        .add(self.source_line.marked())
                        .close_style()
                        .add(self.source_line.suffix())
                })
                .build()
        }
    }

    pub(crate) struct Underline<'codemap> {
        source_line: SourceLine<'codemap>,
        label: &'codemap Label,
        severity: Severity,
    }

    impl<'codemap, 'diagnostic> Underline<'codemap> {
        pub(crate) fn from_message(
            source_line: SourceLine<'codemap>,
            label: &'codemap Label,
            severity: Severity,
        ) -> Underline<'codemap> {
            Underline {
                source_line,
                label,
                severity,
            }
        }

        pub(crate) fn render(&self, config: &impl EmitConfig<'codemap>) -> ComponentNode<'codemap> {
            ComponentNode::build()
                .line({
                    let line_string = self.source_line.location().0.number().to_string();
                    let line_location_prefix = format!(
                        "{} {} ",
                        Pad(' ', line_string.len()),
                        config.gutter_separator()
                    );

                    LineNode::build()
                        .style(config.line_location_style())
                        .add(line_location_prefix)
                        .close_style()
                        .style(config.label_style(self.severity, self.label))
                        .add(Pad(' ', self.source_line.prefix().len()))
                        .add(Pad(
                            config.mark_for(&self.label),
                            self.source_line.marked().len(),
                        ))
                })
                .line({
                    LineNode::build().add_option(&self.label.message, |line, message| {
                        line.style(config.label_style(self.severity, self.label))
                            .add(" ")
                            .add(message)
                    })
                })
                .build()
        }
    }
}

struct DiagnosticWriter<W> {
    writer: W,
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct SourceLine<'codemap> {
    file: &'codemap FileMap,
    label: &'codemap Label,
}

impl<'codemap> SourceLine<'codemap> {
    fn location(&self) -> (LineIndex, ColumnIndex) {
        self.file
            .location(self.label.span.start())
            .expect("location")
    }

    fn filename(&self) -> &'codemap FileName {
        self.file.name()
    }

    fn line_span(&self) -> ByteSpan {
        self.file.line_span(self.location().0).expect("line_span")
    }

    fn prefix(&self) -> &'codemap str {
        self.file
            .src_slice(self.line_span().with_end(self.label.span.start()))
            .expect("line_prefix")
    }

    fn suffix(&self) -> &'codemap str {
        self.file
            .src_slice(self.line_span().with_start(self.label.span.end()))
            .expect("line_suffix")
            .trim_right_matches(|ch| ch == '\r' || ch == '\n')
    }

    fn marked(&self) -> &'codemap str {
        self.file.src_slice(self.label.span).expect("line_marked")
    }
}

impl<W> DiagnosticWriter<W>
where
    W: WriteColor,
{
    fn process_line<'codemap>(
        &mut self,
        file: &'codemap FileMap,
        label: &'codemap Label,
    ) -> SourceLine<'codemap> {
        SourceLine { file, label }
    }

    fn emit<'codemap, 'diagnostic, C: EmitConfig<'codemap>>(
        mut self,
        data: DiagnosticData<'codemap, C>,
    ) -> io::Result<()> {
        let document = DocumentNode::build();

        let header = components::Header::from_diagnostic(&data.diagnostic);

        // error[E0001]: Unexpected type in `+` application
        let mut document = document.add(header.render(&data.config));

        for label in &data.diagnostic.labels {
            match data.codemap.find_file(label.span.start()) {
                None => {
                    let code_line = components::CodeLine::from_message(&label.message);
                    document = document.add(code_line.render(&data.config));
                },
                Some(file) => {
                    let source_line = self.process_line(file, label);

                    // - <test>:3:9
                    let location = components::SourceCodeLocation::from_source_line(source_line);
                    document = document.add(location.render(&data.config));

                    // 3 | (+ test "")
                    let source_code = components::SourceCodeLine::from_message(
                        source_line,
                        label,
                        data.diagnostic.severity,
                    );
                    document = document.add(source_code.render(&data.config));

                    // 3 | (+ test "")     <- prev
                    //   |         ^^      <- this
                    let underline = components::Underline::from_message(
                        source_line,
                        label,
                        data.diagnostic.severity,
                    );
                    document = document.add(underline.render(&data.config));
                },
            }
        }

        document.build().write(&mut self.writer)?;

        Ok(())
    }
}

struct DiagnosticData<'codemap, C> {
    codemap: &'codemap CodeMap,
    diagnostic: &'codemap Diagnostic,
    config: C,
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

        let source = r##"
(define test 123)
(+ test "")
()
"##;
        let file_map = code_map.add_filemap("test".into(), source.to_string());

        let str_start = file_map.byte_index(2.into(), 8.into()).unwrap();
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

        let line_start = file_map.byte_index(2.into(), 0.into()).unwrap();
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
                    - <test>:3:9
                    3 | (+ test "")
                      |         ^^
                     Expected integer but got string
                    - <test>:3:9
                    3 | (+ test "")
                      |         --
                     Expected integer but got string
                    warning: `+` function has no effect unless its result is used
                    - <test>:3:1
                    3 | (+ test "")
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
                println!(
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
            write!(self, "{{/}}")?;
            self.color = ColorSpec::new();
            Ok(())
        }
    }

    #[test]
    fn test_color() {
        assert_eq!(
            emit_with_writer(ColorAccumulator::new()).to_string(),

            // `{---}` in the text below is removed by the normalize() function. It's used to align
            // the text content of lines, so it's inserted below styles in lines above or below it
            normalize(
                r#"
                {fg:Red bold bright}    $$error[E0001]{bold bright}: Unexpected type in `+` application
                                        $$- <test>:3:9
                {fg:Cyan}               $$3 | {/}(+ test {fg:Red}""{/})
                {fg:Cyan}               $$  | {-}{fg:Red}        ^^
                {fg:Red}                $$ Expected integer but got string
                                        $$- <test>:3:9
                {fg:Cyan}               $$3 | {/}(+ test {fg:Cyan}""{/})
                {fg:Cyan}               $$  | {-}        {-------}--
                {fg:Cyan}               $$ Expected integer but got string
                {fg:Yellow bold bright} $$warning{bold bright}: `+` function has no effect unless its result is used
                                        $$- <test>:3:1
                {fg:Cyan}               $$3 | {fg:Yellow}(+ test "")
                {fg:Cyan}               $$  | {fg:Yellow}^^^^^^^^^^^
                                        $$
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
                format!("{{/}}{style}{line}\n", style = style.trim(), line = line)
            })
            .collect()
    }
}
