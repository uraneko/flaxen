// NOTE: styles would be just styled struct and one string
// when a styled needs to be applied, it takes the string and mutates it to its values then it gets
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

#[derive(Default)]
pub struct Styled {
    effects: u8,
    text: Option<Color>,
    background: Option<Color>,
}

#[derive(Default)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn text(&self, styled: &mut String) {
        let color = format!("38;2;{};{};{};", self.r, self.g, self.b);
        styled.push_str(&color)
    }

    fn background(&self, styled: &mut String) {
        let color = format!("48;2;{};{};{};", self.r, self.g, self.b);
        styled.push_str(&color)
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

impl Styled {
    const RESET: u8 = 0; // 0
    const BOLD: u8 = 1; // 1
    const FAINT: u8 = 2; // 2
    const ITALIC: u8 = 4; // 3
    const UNDERLINE: u8 = 8; // 4
    const BLINK: u8 = 16; // 5, 6
    const REVERSE: u8 = 32; // 7
    const CONCEAL: u8 = 64; // 8
    const DBL_UNDERLINE: u8 = 128; // 21

    pub fn new() -> Self {
        Self {
            background: None,
            text: None,
            effects: 0,
        }
    }

    pub fn bold(&mut self) {
        if (self.effects & Styled::BOLD).ne(&0) {
            self.effects &= !Styled::BOLD
        } else {
            self.effects |= Styled::BOLD
        }
    }

    pub fn underline(&mut self) {
        if (self.effects & Styled::UNDERLINE).ne(&0) {
            self.effects &= !Styled::UNDERLINE
        } else {
            self.effects |= Styled::UNDERLINE
        }
    }

    pub fn double_underline(&mut self) {
        if (self.effects & Styled::DBL_UNDERLINE).ne(&0) {
            self.effects &= !Styled::DBL_UNDERLINE
        } else {
            self.effects |= Styled::DBL_UNDERLINE
        }
    }

    pub fn italic(&mut self) {
        if (self.effects & Styled::ITALIC).ne(&0) {
            self.effects &= !Styled::ITALIC
        } else {
            self.effects |= Styled::ITALIC
        }
    }

    pub fn blink(&mut self) {
        if (self.effects & Styled::BLINK).ne(&0) {
            self.effects &= !Styled::BLINK
        } else {
            self.effects |= Styled::BLINK
        }
    }

    pub fn faint(&mut self) {
        if (self.effects & Styled::FAINT).ne(&0) {
            self.effects &= !Styled::FAINT
        } else {
            self.effects |= Styled::FAINT
        }
    }

    pub fn conceal(&mut self) {
        if (self.effects & Styled::CONCEAL).ne(&0) {
            self.effects &= !Styled::CONCEAL
        } else {
            self.effects |= Styled::CONCEAL
        }
    }

    pub fn reverse(&mut self) {
        if (self.effects & Styled::REVERSE).ne(&0) {
            self.effects &= !Styled::REVERSE
        } else {
            self.effects |= Styled::REVERSE
        }
    }

    pub fn reset(&mut self) {
        self.effects &= Self::RESET;
        self.text = None;
        self.background = None;
    }

    pub fn styled(&self) -> String {
        let mut styled = String::from("\x1b[");

        // add effects
        self.bits().iter().for_each(|b| styled += Self::effect(b));

        // add text color
        self.text(&mut styled);

        // add background color
        self.background(&mut styled);

        // clean up the expression
        match styled.remove(styled.len() - 1) {
            '[' => styled += "[0m",
            _ => styled += "m",
        };

        styled
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

    pub fn calibrate(&self, s: &mut String) {
        *s = self.styled();
    }

    fn text(&self, styled: &mut String) {
        if self.text.is_some() {
            self.text.as_ref().unwrap().text(styled);
        }
    }

    fn background(&self, styled: &mut String) {
        if self.background.is_some() {
            self.background.as_ref().unwrap().background(styled);
        }
    }

    pub fn txt(&mut self, color: &[u8; 3]) {
        self.text = Some(Color::new(color[0], color[1], color[2]));
    }

    pub fn bkg(&mut self, color: &[u8; 3]) {
        self.background = Some(Color::new(color[0], color[1], color[2]));
    }
}

pub trait Stylize {
    fn apply(&self, sol: &mut StdoutLock);
}

impl Stylize for String {
    fn apply(&self, sol: &mut StdoutLock) {
        _ = sol.write(&self.as_bytes());
        // unless the whole line is redrawn, the style would not update
        _ = sol.flush();
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
const style_configuration: u32 = 0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled() {}
}
