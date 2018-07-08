use enumflags::BitFlags;
use termcolor::{self, ColorSpec};

#[derive(EnumFlags, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Attributes {
    Bold = 0b001,
    Underline = 0b010,
    Bright = 0b100,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    attributes: BitFlags<Attributes>,
    foreground: Option<ColorValue>,
    background: Option<ColorValue>,
}

impl Style {
    pub fn from_color_spec(spec: ColorSpec) -> Style {
        let mut attributes = BitFlags::empty();

        if spec.bold() {
            attributes |= Attributes::Bold;
        }

        if spec.underline() {
            attributes |= Attributes::Underline;
        }

        if spec.intense() {
            attributes |= Attributes::Bright;
        }

        let foreground = spec.fg().map(|fg| ColorValue(fg.clone()));
        let background = spec.bg().map(|bg| ColorValue(bg.clone()));

        Style {
            attributes,
            foreground,
            background,
        }
    }

    pub fn to_color_spec(&self) -> ColorSpec {
        let mut spec = ColorSpec::new();

        if self.attributes.contains(Attributes::Bold) {
            spec.set_bold(true);
        }

        if self.attributes.contains(Attributes::Underline) {
            spec.set_underline(true);
        }

        if self.attributes.contains(Attributes::Bright) {
            spec.set_intense(true);
        }

        if let Some(ColorValue(fg)) = &self.foreground {
            spec.set_fg(Some(fg.clone()));
        }

        if let Some(ColorValue(bg)) = &self.background {
            spec.set_fg(Some(bg.clone()));
        }

        spec
    }

    pub fn empty() -> Style {
        Style {
            attributes: BitFlags::empty(),
            foreground: None,
            background: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty() && self.foreground.is_none() && self.background.is_none()
    }

    pub fn add_attributes(&self, attributes: Attributes) -> Style {
        self.update(|style| style.attributes |= attributes)
    }

    pub fn remove_attributes(&self, attributes: Attributes) -> Style {
        self.update(|style| style.attributes &= !attributes)
    }

    pub fn fg(&self, color: impl Into<ColorValue>) -> Style {
        self.update(|style| style.foreground = Some(color.into()))
    }

    pub fn clear_fg(&self) -> Style {
        self.update(|style| style.foreground = None)
    }

    pub fn bg(&self, color: impl Into<ColorValue>) -> Style {
        self.update(|style| style.background = Some(color.into()))
    }

    pub fn clear_bg(&self) -> Style {
        self.update(|style| style.background = None)
    }

    pub fn bold(&self) -> Style {
        self.add_attributes(Attributes::Bold)
    }

    pub fn unbold(&self) -> Style {
        self.remove_attributes(Attributes::Bold)
    }

    pub fn underline(&self) -> Style {
        self.add_attributes(Attributes::Underline)
    }

    pub fn remove_underline(&self) -> Style {
        self.remove_attributes(Attributes::Underline)
    }

    pub fn bright(&self) -> Style {
        self.add_attributes(Attributes::Bright)
    }

    pub fn normal_brightness(&self) -> Style {
        self.remove_attributes(Attributes::Bright)
    }

    fn update(&self, f: impl FnOnce(&mut Style)) -> Style {
        let mut style = self.clone();
        f(&mut style);
        style
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ColorValue(termcolor::Color);

impl<'a> From<&'a str> for ColorValue {
    fn from(string: &str) -> ColorValue {
        let string = if string == "blue" {
            // Blue looks terrible on Windows
            if cfg!(windows) {
                "cyan"
            } else {
                "blue"
            }
        } else {
            string
        };

        ColorValue(string.parse().unwrap())
    }
}

impl From<termcolor::Color> for ColorValue {
    fn from(color: termcolor::Color) -> ColorValue {
        ColorValue(color)
    }
}
