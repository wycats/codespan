use codespan::FileName;
use std::fmt;
use {ColorValue, Label, LabelStyle, Severity, Style};

pub trait EmitConfig<'codemap> {
    fn filename(&self, name: &'codemap FileName, f: &mut fmt::Formatter) -> fmt::Result;

    // &'static str because single-width output can be multiple characters (emoji, e.g.)
    fn mark_for(&self, label: &Label) -> &'static str;
    fn primary_underline(&self) -> char;
    fn secondary_underline(&self) -> char;
    fn gutter_separator(&self) -> &'static str;

    fn line_location_style(&self) -> Style;
    fn severity_style(&self, severity: Severity) -> Style;
    fn highlight_style(&self) -> Style;
    fn label_style(&self, severity: Severity, label: &Label) -> Style;

    fn severity_color(&self, severity: Severity) -> ColorValue;
}

pub(crate) struct DefaultConfig;

impl<'codemap> EmitConfig<'codemap> for DefaultConfig {
    fn filename(&self, filename: &'codemap FileName, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", filename)
    }

    fn mark_for(&self, label: &Label) -> &'static str {
        match label.style {
            LabelStyle::Primary => "^",
            LabelStyle::Secondary => "-",
        }
    }

    fn primary_underline(&self) -> char {
        '^'
    }

    fn secondary_underline(&self) -> char {
        '-'
    }

    fn gutter_separator(&self) -> &'static str {
        "|"
    }

    fn line_location_style(&self) -> Style {
        Style::empty().fg("blue")
    }

    fn severity_style(&self, severity: Severity) -> Style {
        Style::empty().fg(self.severity_color(severity))
    }

    fn highlight_style(&self) -> Style {
        Style::empty().bold().bright()
    }

    fn label_style(&self, severity: Severity, label: &Label) -> Style {
        match label.style {
            LabelStyle::Primary => self.severity_style(severity),
            LabelStyle::Secondary => Style::empty().fg("blue"),
        }
    }

    fn severity_color(&self, severity: Severity) -> ColorValue {
        match severity {
            Severity::Bug | Severity::Error => "red".into(),
            Severity::Warning => "yellow".into(),
            Severity::Note => "green".into(),
            Severity::Help => "cyan".into(),
        }
    }
}
