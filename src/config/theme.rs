use serde::{Deserialize, Serialize};

const ESCAPE_CODE: &str = "\x1b";

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigTheme {
    pub executable: StylePair,
    pub path: StylePair,

    pub flag: StylePair,
    pub arg: StylePair,
    pub text: StylePair,

    pub console_main: StylePair,
    pub console_secondary: StylePair,
    pub error: StylePair,
}

impl ConfigTheme {
    pub fn generate_escape_sequences(&mut self) {
        self.executable.generate_escape_sequences();
        self.path.generate_escape_sequences();

        self.flag.generate_escape_sequences();
        self.arg.generate_escape_sequences();
        self.text.generate_escape_sequences();

        self.console_main.generate_escape_sequences();
        self.console_secondary.generate_escape_sequences();
        self.error.generate_escape_sequences();
    }
}

impl Default for ConfigTheme {
    fn default() -> Self {
        use Color as C;
        use Formatter as F;
        Self {
            executable: StylePair::new(
                Style::new(vec![], C::Green, None),
                Style::new(vec![], C::Green, Some(C::White)),
            ),
            path: StylePair::new(
                Style::new(vec![], C::Blue, None),
                Style::new(vec![], C::Blue, Some(C::Red)),
            ),

            flag: StylePair::new(
                Style::new(vec![], C::Magenta, None),
                Style::new(vec![], C::Magenta, Some(C::Cyan)),
            ),
            arg: StylePair::new(
                Style::new(vec![], C::Yellow, None),
                Style::new(vec![], C::Yellow, Some(C::Blue)),
            ),
            text: StylePair::new(
                Style::new(vec![], C::White, None),
                Style::new(vec![], C::White, Some(C::Black)),
            ),

            console_main: StylePair::new(
                Style::new(vec![F::Bold], C::Yellow, None),
                Style::new(vec![F::Bold], C::Yellow, Some(C::Red)),
            ),
            console_secondary: StylePair::new(
                Style::new(vec![F::Italic], C::Cyan, None),
                Style::new(vec![F::Italic], C::Cyan, Some(C::White)),
            ),
            error: StylePair::new(
                Style::new(vec![F::Bold], C::Red, None),
                Style::new(vec![F::Bold], C::Red, Some(C::Yellow)),
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StylePair {
    pub normal: Style,
    pub highlighted: Style,
}

impl StylePair {
    pub fn new(normal: Style, highlighted: Style) -> Self {
        Self {
            normal,
            highlighted,
        }
    }

    pub fn generate_escape_sequences(&mut self) {
        self.highlighted.generate_escape_sequence();
        self.normal.generate_escape_sequence();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Style {
    modifiers: Vec<Formatter>,
    foreground: Color,
    background: Option<Color>,

    #[serde(skip)]
    pub escape_sequence: String,
}

impl Style {
    pub fn new(modifiers: Vec<Formatter>, foreground: Color, background: Option<Color>) -> Self {
        Self {
            modifiers,
            foreground,
            background,
            escape_sequence: String::new(),
        }
    }

    pub fn generate_escape_sequence(&mut self) {
        let background = if let Some(color) = &self.background {
            format!(";{}", color.background_code())
        } else {
            "".to_string()
        };

        let codes = self
            .modifiers
            .iter()
            .map(|f| f.generate_code())
            .collect::<Vec<_>>()
            .join(";");

        self.escape_sequence = if !codes.is_empty() {
            format!(
                "{}[{};{}{}m",
                ESCAPE_CODE,
                codes,
                self.foreground.foreground_code(),
                background
            )
        } else {
            format!(
                "{}[{}{}m",
                ESCAPE_CODE,
                self.foreground.foreground_code(),
                background
            )
        };
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Formatter {
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Hidden,
    Strike,
}

impl Formatter {
    pub fn generate_code(&self) -> &str {
        match self {
            Formatter::Bold => "1",
            Formatter::Dim => "2",
            Formatter::Italic => "3",
            Formatter::Underline => "4",
            Formatter::Blink => "5",
            Formatter::Reverse => "7",
            Formatter::Hidden => "8",
            Formatter::Strike => "9",
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Color {
    pub fn foreground_code(&self) -> &str {
        match self {
            Color::Black => "30",
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Blue => "34",
            Color::Magenta => "35",
            Color::Cyan => "36",
            Color::White => "37",
        }
    }

    pub fn background_code(&self) -> &str {
        match self {
            Color::Black => "40",
            Color::Red => "41",
            Color::Green => "42",
            Color::Yellow => "43",
            Color::Blue => "44",
            Color::Magenta => "45",
            Color::Cyan => "46",
            Color::White => "47",
        }
    }
}
