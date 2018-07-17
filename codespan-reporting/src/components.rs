use emitter::DiagnosticData;
use models;
use models::severity;
use render_tree::*;

pub(crate) struct Diagnostic;

impl<'args> RenderComponent<'args> for Diagnostic {
    type Args = DiagnosticData<'args>;

    fn render(&self, data: DiagnosticData<'args>) -> Document {
        let header = models::Header::new(&data.diagnostic);

        tree! {
            <section name={severity(&data.diagnostic)} {
                <Header {header}>
                <Body {data}>
            }>
        }
    }
}

pub(crate) struct Header;

impl<'args> RenderComponent<'args> for Header {
    type Args = models::Header<'args>;

    fn render(&self, header: models::Header<'args>) -> Document {
        tree! {
            <section name="header" {
                <line {
                    <section name="primary" {
                        // error
                        {header.severity()}
                        // [E0001]
                        {IfSome(&header.code().map(|code| tree! { "[" {code} "]" }))}
                    }>
                    ": "
                    // Unexpected type in `+` application
                    {header.message()}
                }>
            }>
        }
    }
}

pub(crate) struct Body;

impl<'args> RenderComponent<'args> for Body {
    type Args = DiagnosticData<'args>;

    fn render(&self, data: DiagnosticData<'args>) -> Document {
        Each(&data.diagnostic.labels, |label| {
            match data.codemap.find_file(label.span.start()) {
                None => {
                    tree! { <CodeLine {models::Message::new(&label.message)}> }
                },
                Some(file) => {
                    let source_line = models::SourceLine::new(file, label);
                    let labelled_line = models::LabelledLine::new(source_line, label);

                    tree! {
                        // - <test>:2:9
                        <SourceCodeLocation {source_line}>

                        // 2 | (+ test "")
                        //   |         ^^
                        <SourceCodeLine {labelled_line}>
                    }
                },
            }
        }).into_fragment()
    }
}

pub(crate) struct CodeLine;

impl<'args> RenderComponent<'args> for CodeLine {
    type Args = models::Message<'args>;

    fn render(&self, message: models::Message<'args>) -> Document {
        tree! {
            <section name="code-line" {
                <line {
                    "- " {IfSome(message.message())}
                }>
            }>
        }
    }
}

pub(crate) struct SourceCodeLocation;

impl<'args> RenderComponent<'args> for SourceCodeLocation {
    type Args = models::SourceLine<'args>;

    fn render(&self, source_line: models::SourceLine) -> Document {
        let (line, column) = source_line.location();
        let filename = source_line.filename().to_string();

        tree! {
            <section name="source-code-location" {
                <line {
                    // - <test>:3:9
                    "- " {filename} ":" {line.number()}
                    ":" {column.number()}
                }>
            }>
        }
    }
}

pub(crate) struct SourceCodeLine;

impl<'args> RenderComponent<'args> for SourceCodeLine {
    type Args = models::LabelledLine<'args>;

    fn render(&self, model: models::LabelledLine<'args>) -> Document {
        let source_line = model.source_line();

        let message = model.message().map(|message| tree!({" "} {message}));

        tree! {
            <line {
                <section name="gutter" {
                    {source_line.line_number()}
                    " | "
                }>

                <section name="before-marked" {
                    {source_line.before_marked()}
                }>

                <section name={model.style()} {
                    {model.source_line().marked()}
                }>

                <section name="after-marked" {
                    {source_line.after_marked()}
                }>
            }>

            <line {
                <section name="underline" {
                    <section name="gutter" {
                        {repeat(" ", model.source_line().line_number_len())}
                        " | "
                    }>

                    {repeat(" ", model.source_line().before_marked().len())}
                    <section name={model.style()} {
                        {repeat(model.mark(), model.source_line().marked().len())}
                        {IfSome(&message)}
                    }>
                }>
            }>
        }
    }
}
