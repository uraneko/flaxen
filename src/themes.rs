// when a style needs to be applied, it takes the string and mutates it to its values then it gets
// sent over to the event queue to be applied to text

use std::io::StdoutLock;
use std::io::Write;

// NOTE: should create a stylegraph that takes styles
// styles are applied according to stylegraphs
// stylegraphs define rules for which styles apply to which text
// the rules are based on text tokens' attributes
// whether a token includes or excludes (starts, ends or contains) a certain pattern
// the position of the token in the text
// or can take individual chars instead of whole tokens

#[derive(Debug, Default)]
pub struct Style {
    effects: u8,
    text: Option<Color>,
    background: Option<Color>,
}

#[derive(Default, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn text(&self, style: &mut String) {
        let color = format!("38;2;{};{};{};", self.r, self.g, self.b);
        style.push_str(&color)
    }

    fn background(&self, style: &mut String) {
        let color = format!("48;2;{};{};{};", self.r, self.g, self.b);
        style.push_str(&color)
    }

    // fn red(&mut self, r: u8) {
    //     self.r = r;
    // }
    //
    // fn green(&mut self, g: u8) {
    //     self.g = g;
    // }
    //
    // fn blue(&mut self, b: u8) {
    //     self.b = b;
    // }
}

impl Style {
    const RESET: u8 = 0; // 0
    const BOLD: u8 = 1; // 1
    const FAINT: u8 = 2; // 2
    const ITALIC: u8 = 4; // 3
    const UNDERLINE: u8 = 8; // 4
    const BLINK: u8 = 16; // 5, 6
    const REVERSE: u8 = 32; // 7
    const CONCEAL: u8 = 64; // 8
    const DBL_UNDERLINE: u8 = 128; // 21

    pub fn new(/* name: &str */) -> Self {
        Self {
            background: None,
            text: None,
            effects: 0,
        }
    }

    pub fn bold(mut self) -> Self {
        if (self.effects & Style::BOLD).ne(&0) {
            self.effects &= !Style::BOLD
        } else {
            self.effects |= Style::BOLD
        }

        self
    }

    pub fn underline(mut self) -> Self {
        if (self.effects & Style::UNDERLINE).ne(&0) {
            self.effects &= !Style::UNDERLINE
        } else {
            self.effects |= Style::UNDERLINE
        }

        self
    }

    pub fn double_underline(mut self) -> Self {
        if (self.effects & Style::DBL_UNDERLINE).ne(&0) {
            self.effects &= !Style::DBL_UNDERLINE
        } else {
            self.effects |= Style::DBL_UNDERLINE
        }

        self
    }

    pub fn italic(mut self) -> Self {
        if (self.effects & Style::ITALIC).ne(&0) {
            self.effects &= !Style::ITALIC
        } else {
            self.effects |= Style::ITALIC
        }

        self
    }

    pub fn blink(mut self) -> Self {
        if (self.effects & Style::BLINK).ne(&0) {
            self.effects &= !Style::BLINK
        } else {
            self.effects |= Style::BLINK
        }

        self
    }

    pub fn faint(mut self) -> Self {
        if (self.effects & Style::FAINT).ne(&0) {
            self.effects &= !Style::FAINT
        } else {
            self.effects |= Style::FAINT
        }

        self
    }

    pub fn conceal(mut self) -> Self {
        if (self.effects & Style::CONCEAL).ne(&0) {
            self.effects &= !Style::CONCEAL
        } else {
            self.effects |= Style::CONCEAL
        }

        self
    }

    pub fn reverse(mut self) -> Self {
        if (self.effects & Style::REVERSE).ne(&0) {
            self.effects &= !Style::REVERSE
        } else {
            self.effects |= Style::REVERSE
        }

        self
    }

    pub fn reset(mut self) -> Self {
        self.effects &= Self::RESET;
        self.text = None;
        self.background = None;

        self
    }

    pub fn style(&self) -> String {
        let mut style = String::from("\x1b[");

        // add effects
        self.bits().iter().for_each(|b| style += Self::effect(b));

        // add text color
        self.text(&mut style);

        // add background color
        self.background(&mut style);

        // clean up the expression
        match style.remove(style.len() - 1) {
            '[' => style += "[0m",
            _ => style += "m",
        };

        style
    }

    fn bits(&self) -> [u8; 8] {
        [
            self.effects & Self::DBL_UNDERLINE,
            self.effects & Self::CONCEAL,
            self.effects & Self::REVERSE,
            self.effects & Self::BLINK,
            self.effects & Self::UNDERLINE,
            self.effects & Self::ITALIC,
            self.effects & Self::FAINT,
            self.effects & Self::BOLD,
        ]
    }

    fn effect<'a>(effect: &u8) -> &'a str {
        match effect {
            0 => "",
            1 => "1;",
            2 => "2;",
            4 => "3;",
            8 => "4;",
            16 => "5;",
            32 => "7;",
            64 => "8;",
            128 => "21;",
            _ => unreachable!(
                "there is no effect with such an index, the index must be: 0 =< idx < 8 "
            ),
        }
    }

    // dumps the current style values into a string
    pub fn calibrate(&self, s: &mut String) {
        *s = self.style();
    }

    fn text(&self, style: &mut String) {
        if self.text.is_some() {
            self.text.as_ref().unwrap().text(style);
        }
    }

    fn background(&self, style: &mut String) {
        if self.background.is_some() {
            self.background.as_ref().unwrap().background(style);
        }
    }

    // changes the style text color to the provided rgb value
    pub fn txt(mut self, color: &[u8; 3]) -> Self {
        self.text = Some(Color::new(color[0], color[1], color[2]));

        self
    }

    // changes the style background color to the provided rgb value
    pub fn bkg(mut self, color: &[u8; 3]) -> Self {
        self.background = Some(Color::new(color[0], color[1], color[2]));

        self
    }
}

/// can only have one combination that results in the same sum
/// 0 means reset all
/// 1 means bold
/// 2 means underline
/// 4 means double underline
/// 8 means italic
/// 16 means reverse
/// 32 means conceal
/// 64 means blink
/// 128 means faint
/// the greatest effects config value possible is 255

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style() {}
}

struct Token {}

use std::collections::HashMap;
use std::ops::Range;

pub trait Theme {
    fn styles(&mut self, styles: Vec<Style>) -> &str {
        "\x1b[0m"
    }

    // usually used on input
    // indicates the style of the first word in an input text
    fn cmd(&mut self, style: &Style) -> &str {
        "\x1b[0m"
    }

    // usually used on input
    // describes the style of a word that starts with '--' or '-' in an input text
    fn param(&mut self, style: &Style) -> &str {
        "\x1b[0m"
    }

    // usually used on input
    // describes the style of the arg that comes after a param
    fn arg(&mut self, style: &Style) -> &str {
        "\x1b[0m"
    }

    // usually used on input
    // describes the style of a flag, which represents a param with no arg
    fn flag(&mut self, style: &Style) -> &str {
        "\x1b[0m"
    }

    // describes the style of an object border
    fn border(&mut self, style: &Style) -> &str {
        "\x1b[0m"
    }

    fn custom(&mut self, style: &Style) -> &str {
        "\x1b[0m"
    }

    // describes the style of a whole body of text
    fn text(&mut self, style: &Style) -> &str {
        "\x1b[0m"
    }

    // gets called inside an Event that colors all objects
    // applies what the user chooses to apply for that instance of a type from the function above
    // the only method of this trait that is required (no default impl)
    // NOTE: gets called inside term.process() right before term.clear() in preparation for
    // rendering
    fn theme(&mut self, map: &HashMap<&str, Style>);
}

// NOTE: Theme trait usage:
// 1. implement the methods that you want to implement
//    - ok, but what do these methods do???
// 2. implement the theme method which calls the methods you choose to call from amongst the
//    implemented methods of the trait
// 3. impl theming Events fire method which calls the theme method of the Theme trait on all
//    objects in the active Term
//    the Theme trait should take a generic so that external crates can implement it, just like
//    Events trait
//    in the theme() method you take your styles and iterate through the going to be rendered
//    buffer of every object and you make conditions for when to apply which style
//    technically, only the theme method is needed and should take a hashmap / vec if styles

// a theme being implemented means 2 things
// border styles and value styles
// for a container there is only border
// for a text there is both border and value
//
// NOTE: need a way in which a theme is applied to a whole Object
// and if a part needs to be custom styled we write a custom style and only that part gets custom
// styled
// NOTE: i still cant figure out how to pull out an atomically customizable theme
// so for now ill make the input command value custom and everything else gets to be under one
// theme
