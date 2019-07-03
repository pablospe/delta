use std::fmt::Write;
use std::str::FromStr;
// TODO: Functions in this module should return Result and use ? syntax.

use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

use crate::state_machine::State;

pub const LIGHT_THEMES: [&str; 4] = [
    "GitHub",
    "Monokai Extended Light",
    "OneHalfLight",
    "ansi-light",
];

pub fn is_light_theme(theme: &str) -> bool {
    LIGHT_THEMES.contains(&theme)
}

const LIGHT_THEME_PLUS_COLOR: Color = Color {
    r: 0xd0,
    g: 0xff,
    b: 0xd0,
    a: 0xff,
};

const LIGHT_THEME_MINUS_COLOR: Color = Color {
    r: 0xff,
    g: 0xd0,
    b: 0xd0,
    a: 0xff,
};

const DARK_THEME_PLUS_COLOR: Color = Color {
    r: 0x01,
    g: 0x3B,
    b: 0x01,
    a: 0xff,
};

const DARK_THEME_MINUS_COLOR: Color = Color {
    r: 0x3F,
    g: 0x00,
    b: 0x01,
    a: 0xff,
};

pub struct Config<'a> {
    theme: &'a Theme,
    plus_color: Color,
    minus_color: Color,
    pub syntax_set: &'a SyntaxSet,
    width: Option<usize>,
    highlight_removed: bool,
    pub pager: &'a str,
}

pub fn get_config<'a>(
    syntax_set: &'a SyntaxSet,
    theme: &Option<String>,
    theme_set: &'a ThemeSet,
    user_requests_theme_for_light_terminal_background: bool,
    plus_color_str: &Option<String>,
    minus_color_str: &Option<String>,
    highlight_removed: bool,
    width: Option<usize>,
) -> Config<'a> {
    let theme_name = match theme {
        Some(ref theme) => theme,
        None => match user_requests_theme_for_light_terminal_background {
            true => "GitHub",
            false => "Monokai Extended",
        },
    };
    let minus_color = minus_color_str
        .as_ref()
        .and_then(|s| Color::from_str(s).ok());
    let plus_color = plus_color_str
        .as_ref()
        .and_then(|s| Color::from_str(s).ok());

    let is_light_theme = LIGHT_THEMES.contains(&theme_name);

    Config {
        theme: &theme_set.themes[theme_name],
        plus_color: plus_color.unwrap_or_else(|| {
            if is_light_theme {
                LIGHT_THEME_PLUS_COLOR
            } else {
                DARK_THEME_PLUS_COLOR
            }
        }),
        minus_color: minus_color.unwrap_or_else(|| {
            if is_light_theme {
                LIGHT_THEME_MINUS_COLOR
            } else {
                DARK_THEME_MINUS_COLOR
            }
        }),
        width: width,
        highlight_removed: highlight_removed,
        syntax_set: &syntax_set,
        pager: "less",
    }
}

/// Write line to buffer with color escape codes applied.
pub fn paint_line(
    mut line: String,
    state: &State,
    syntax: &SyntaxReference,
    config: &Config,
    buf: &mut String,
) {
    let background_color: Option<Color>;
    let apply_syntax_highlighting: bool;

    match state {
        State::HunkZero => {
            background_color = None;
            apply_syntax_highlighting = true;
        }
        State::HunkMinus => {
            background_color = Some(config.minus_color);
            apply_syntax_highlighting = config.highlight_removed;
            line = line[1..].to_string();
            buf.push_str(" ");
        }
        State::HunkPlus => {
            background_color = Some(config.plus_color);
            apply_syntax_highlighting = true;
            line = line[1..].to_string();
            buf.push_str(" ");
        }
        _ => panic!("Invalid state: {:?}", state),
    }
    match config.width {
        Some(width) => {
            if line.len() < width {
                line = format!("{}{}", line, " ".repeat(width - line.len()));
            }
        }
        _ => (),
    }
    paint_text(
        line,
        syntax,
        background_color,
        config,
        apply_syntax_highlighting,
        buf,
    );
}

// TODO: If apply_syntax_highlighting is false, then don't do
// operations related to syntax highlighting.

pub fn paint_text(
    text: String,
    syntax: &SyntaxReference,
    background_color: Option<Color>,
    config: &Config,
    apply_syntax_highlighting: bool,
    buf: &mut String,
) {
    let mut highlighter = HighlightLines::new(syntax, config.theme);

    match background_color {
        Some(background_color) => {
            write!(
                buf,
                "\x1b[48;2;{};{};{}m",
                background_color.r, background_color.g, background_color.b
            )
            .unwrap();
        }
        None => (),
    }
    let ranges: Vec<(Style, &str)> = highlighter.highlight(&text, &config.syntax_set);
    paint_ranges(&ranges[..], None, apply_syntax_highlighting, buf);
}

/// Based on as_24_bit_terminal_escaped from syntect
fn paint_ranges(
    foreground_style_ranges: &[(Style, &str)],
    background_color: Option<Color>,
    apply_syntax_highlighting: bool,
    buf: &mut String,
) -> () {
    for &(ref style, text) in foreground_style_ranges.iter() {
        paint(
            text,
            if apply_syntax_highlighting {
                Some(style.foreground)
            } else {
                None
            },
            background_color,
            buf,
        );
    }
}

/// Write text to buffer with color escape codes applied.
fn paint(
    text: &str,
    foreground_color: Option<Color>,
    background_color: Option<Color>,
    buf: &mut String,
) -> () {
    match background_color {
        Some(background_color) => {
            write!(
                buf,
                "\x1b[48;2;{};{};{}m",
                background_color.r, background_color.g, background_color.b
            )
            .unwrap();
        }
        None => (),
    }
    match foreground_color {
        Some(foreground_color) => {
            write!(
                buf,
                "\x1b[38;2;{};{};{}m{}",
                foreground_color.r, foreground_color.g, foreground_color.b, text
            )
            .unwrap();
        }
        None => {
            write!(buf, "{}", text).unwrap();
        }
    }
}
