#![allow(non_snake_case)]

use emitter::DiagnosticData;
use models;
use models::severity;
use render_tree::*;

pub(crate) fn Diagnostic<'args>(data: DiagnosticData<'args>, into: Document) -> Document {
    let header = models::Header::new(&data.diagnostic);

    into.add(tree! {
        <section name={severity(&data.diagnostic)} {
            <Header arg={header}>
            <Body arg={data}>
        }>
    })
}

pub(crate) fn Header<'args>(header: models::Header<'args>, into: Document) -> Document {
    into.add(tree! {
        <section name="header" {
            <Line as {
                <section name="primary" {
                    // error
                    {header.severity()}
                    // [E0001]
                    {IfSome(header.code(), |code| tree! { "[" {code} "]" })}
                }>
                ": "
                // Unexpected type in `+` application
                {header.message()}
            }>
        }>
    })
}

pub(crate) fn Body<'args>(data: DiagnosticData<'args>, mut into: Document) -> Document {
    for label in &data.diagnostic.labels {
        match data.codemap.find_file(label.span.start()) {
            None => {
                into = into.add(tree! { <CodeLine args={models::Message::new(&label.message)}> })
            },
            Some(file) => {
                let source_line = models::SourceLine::new(file, label);
                let labelled_line = models::LabelledLine::new(source_line, label);

                into = into.add(tree! {
                    // - <test>:2:9
                    <SourceCodeLocation args={source_line}>

                    // 2 | (+ test "")
                    //   |         ^^
                    <SourceCodeLine arg={labelled_line}>
                })
            },
        }
    }

    into
}

pub(crate) fn CodeLine<'args>(message: models::Message<'args>, into: Document) -> Document {
    into.add(tree! {
        <section name="code-line" {
            <Line as {
                "- " {SomeValue(message.message())}
            }>
        }>
    })
}

pub(crate) fn SourceCodeLocation(source_line: models::SourceLine, into: Document) -> Document {
    let (line, column) = source_line.location();
    let filename = source_line.filename().to_string();

    into.add(tree! {
        <section name="source-code-location" {
            <Line as {
                // - <test>:3:9
                "- " {filename} ":" {line.number()}
                ":" {column.number()}
            }>
        }>
    })
}

pub(crate) fn SourceCodeLine<'args>(
    model: models::LabelledLine<'args>,
    into: Document,
) -> Document {
    let source_line = model.source_line();

    into.add(tree! {
        <Line as {
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

        <Line as {
            <section name="underline" {
                <section name="gutter" {
                    {repeat(" ", model.source_line().line_number_len())}
                    " | "
                }>

                {repeat(" ", model.source_line().before_marked().len())}
                <section name={model.style()} {
                    {repeat(model.mark(), model.source_line().marked().len())}
                    {IfSome(model.message(), |message| tree!({" "} {message}))}
                }>
            }>
        }>
    })
}
