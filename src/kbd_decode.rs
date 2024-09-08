use std::collections::VecDeque;
use std::io::Error as er;
use std::io::Read;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub enum Char {
    CC(CC),
    Char(char),
}

impl Default for Char {
    fn default() -> Self {
        Self::Char(' ')
    }
}

impl Char {
    fn from_ascii(byte: u8) -> Self {
        Char::Char(byte as char)
    }

    fn from_cc(byte: u8) -> Self {
        Char::CC(match byte {
            9 => CC::TAB,
            13 => CC::CR,
            27 => CC::ESC,
            127 => CC::BS,
            val => unreachable!("unexpected byte value: '{}'. not a control char", val),
        })
    }

    fn from_ctrl_ascii(byte: u8) -> Self {
        Char::Char(match byte {
            0 => '@',
            1..=26 => (byte + 96) as char,
            27..30 => (byte + 64) as char,
            30 => (byte + 24) as char,
            31 => '_',
            val => unreachable!("expected byte between 0 and 31, got '{}' instead", val),
        })
    }

    /// can't fail
    fn from_utf8(bytes: &[u8]) -> Self {
        Char::Char(std::str::from_utf8(bytes).unwrap().chars().last().unwrap())
    }

    fn from_arrow_key(byte: u8) -> Self {
        Char::CC(match byte {
            65 => CC::Up,
            66 => CC::Down,
            67 => CC::Right,
            68 => CC::Left,
            _ => unreachable!("already asserted this byte to be between 65 and 68"),
        })
    }

    // fn keys that generate 3 bytes long input
    fn from_fn_key3(byte: u8) -> Self {
        Char::CC(match byte {
            80 => CC::F1,
            81 => CC::F2,
            82 => CC::F3,
            83 => CC::F4,
            _ => unreachable!("already asserted this byte to be between 80 and 83"),
        })
    }

    fn from_cc_extra(byte: u8) -> Self {
        Char::CC(match byte {
            51 => CC::Insert,
            52 => CC::End,
            54 => CC::PageDown,
            53 => CC::PageUp,
            49 => CC::Home,
            _ => unreachable!("already asserted this byte to be between 80 and 83"),
        })
    }

    fn from_fn_key5(byte2: u8, byte3: u8) -> Self {
        Char::CC(match byte3 {
            53 if byte2 == 49 => CC::F5,
            55 if byte2 == 49 => CC::F6,
            56 if byte2 == 49 => CC::F7,
            57 if byte2 == 49 => CC::F8,
            48 if byte2 == 50 => CC::F9,
            49 if byte2 == 50 => CC::F10,
            51 if byte2 == 50 => CC::F11,
            52 if byte2 == 50 => CC::F12,
            _ => unreachable!(
                "already asserted these bytes, 
                if you really do reach here, it's because
                i let some cases get away in my assertions\r\n
                Please open an issue here \"https://github.com/uraneko/ragout\" 
                so that i can fix the bug"
            ),
        })
    }

    // fn from_6b_with_mods(bytes: &[u8]) -> Self {}

    fn from_utf81(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 1);

        Self::from_utf8(bytes)
    }
    fn from_utf82(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 2);

        Self::from_utf8(bytes)
    }
    fn from_utf83(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 3);

        Self::from_utf8(bytes)
    }
    fn from_utf84(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 4);

        Self::from_utf8(bytes)
    }
}

#[derive(Debug, PartialEq)]
pub enum CC {
    // ascii
    BS = 127,
    TAB = 9,
    CR = 13,
    ESC = 27,
    // not ascii
    Up,
    Down,
    Right,
    Left,
    Insert, // Del
    Home,
    End,
    PageUp,
    PageDown,
    // fn keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

#[derive(Default, Clone, PartialEq)]
pub struct Modifiers(pub u8);

impl std::fmt::Display for Modifiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_text(),)
    }
}

impl std::fmt::Debug for Modifiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_text())
    }
}

const SUPER: u8 = 0x01;
const CONTROL: u8 = 0x02;
const ALT: u8 = 0x04;
const SHIFT: u8 = 0x08;
const NONE: u8 = 0x0;

impl Modifiers {
    fn from_byte(byte: u8) -> Self {
        assert!(byte < 16);

        Self(byte)
    }

    fn as_text(&self) -> &str {
        match self.0 {
            0 => "NONE",
            1 => "SUPER",
            2 => "CONTROL",
            3 => "CONTROL_SUPER",
            4 => "ALT",
            5 => "SUPER_ALT",
            6 => "CONTROL_ALT",
            7 => "CONTROL_ALT_SUPER",
            8 => "SHIFT",
            9 => "SUPER_SHIFT",
            10 => "CONTROL_SHIFT",
            11 => "CONTROL_SUPER_SHIFT",
            12 => "SHIFT_ALT",
            13 => "SUPER_SHIFT_ALT",
            14 => "CONTROL_SHIFT_ALT",
            15 => "CONTROL_SHIFT_ALT_SUPER",

            _ => unreachable!(
                "basic values are only control(2), shift(8), alt(4) and super(1)
                    \r\n you did not present a valid combination of those values"
            ),
        }
    }

    fn from_or(bytes: &[u8]) -> Self {
        let modifier = Self(bytes.into_iter().fold(0, |acc, b| acc | b));
        assert!(modifier.0 < 16);

        modifier
    }

    fn or(&mut self, byte: u8) {
        assert!(self.0 | byte < 16);

        self.0 |= byte;
    }

    // 6 bytes escape sequence modifiers identification
    // the modifier byte is bytes[4] (the 5th byte)
    // in the inputted escape sequence
    fn from_raw67(byte: u8) -> Self {
        Self(match byte {
            50 => SHIFT,
            51 => ALT,
            52 => SHIFT | ALT,
            53 => CONTROL,
            54 => CONTROL | SHIFT,
            55 => CONTROL | ALT,
            56 => CONTROL | SHIFT | ALT,
            57 => SUPER,
            _ => unreachable!(
                "What manner of key combinations did 
                you input to get this byte, sir?",
            ),
        })
    }

    // 7 bytes escape sequence modifiers identification
    fn from_super7(byte: u8) -> Self {
        Self(match byte {
            48 => 9,
            49 => 5,
            50 => 13,
            51 => 3,
            52 => 11,
            53 => 7,
            54 => 15,
            val => unreachable!("all mods handled, who are you? {}", val),
        })
    }
}

#[derive(Debug)]
pub struct KbdEvent {
    pub char: Char,
    pub modifiers: Modifiers,
}

impl Default for KbdEvent {
    fn default() -> Self {
        Self {
            modifiers: Modifiers(0x0),
            char: Char::CC(CC::ESC),
        }
    }
}

mod utf8_decoder {
    use super::*;

    /// the buffer i use to get data is an 8 bytes long array [u8; 8] initially filled with 0s
    /// this function filters out the excess NIL bytes that were not filled
    pub(super) fn filter_nil(bytes: &[u8; 8]) -> Vec<u8> {
        bytes
            .into_iter()
            .enumerate()
            .fold(Vec::new(), |mut acc, (idx, b)| {
                if idx == 0 || (*b != 0 && bytes[idx - 1] != 0) {
                    acc.push(*b);
                    acc
                } else {
                    acc
                }
            })
    }

    /// the utf 8 value is only 1 byte long and can only have a 0 for its highest bit
    /// this effectively means it's an ascii value; either char or control char
    /// three cases to this:
    /// - ascii char
    /// - ascii control char (cr, bs, esc or tab)
    /// - ascii char with ctrl modifier
    fn decode_1_byte(byte: u8, ke: &mut KbdEvent) {
        match byte {
            0..=31 | 127 => match byte {
                9 | 13 | 27 | 127 => ke.char = Char::from_cc(byte),
                _val => {
                    *ke = KbdEvent {
                        char: Char::from_ctrl_ascii(byte),
                        modifiers: Modifiers(CONTROL),
                    }
                }
            },
            32..=126 => ke.char = Char::from_ascii(byte),
            _ => unreachable!("ascii are limited by 7 bits a byte, so from 0 to 127"),
        }
    }

    /// three cases
    /// - modifier byte (alt) followed by an ascii value
    /// - 2 byte utf8 char
    /// - modifier byte (alt) followed by modifier (ctrl) altered ascii byte/value
    /// easy problem, since, in utf8 rules, a 2 bytes utf8 value MUST have a first byte that has
    /// 110 as its highest 3 bits and a second byte that has 10 as its highest 2 bits
    fn decode_2_bytes(bytes: &[u8], ke: &mut KbdEvent) {
        assert_eq!(bytes.len(), 2);
        // not the first byte of a 2 bytes utf8 so not a utf8 value
        // by elimination, this is an alt + ascii event
        // FIXME: these utf8 conditions are wrong;
        // if 1 of the 2 highest bits matches it wont give a 0,
        // despite only one bit matching
        // FIX
        if is_utf82(bytes[0], bytes[1]) {
            ke.char = Char::from_utf8(bytes);
        } else {
            assert_eq!(27, bytes[0]);
            decode_1_byte(bytes[1], ke);
            ke.modifiers.or(ALT);
        }
    }

    // cases
    // - 3 bytes utf8
    // - arrow key
    // - fn keys
    fn decode_3_bytes(bytes: &[u8], ke: &mut KbdEvent) {
        assert_eq!(bytes.len(), 3);
        // valid '3 bytes' utf8 first byte
        if is_utf83(bytes[0], bytes[1], bytes[2]) {
            ke.char = Char::from_utf8(bytes);
        } else {
            assert_eq!(bytes[0], 27);
            match bytes[1] {
                91 => {
                    if bytes[2] == 90 {
                        ke.modifiers.or(SHIFT);
                        ke.char = Char::CC(CC::TAB);
                    } else {
                        assert!(bytes[2] >= 65 && bytes[2] <= 68);
                        ke.char = Char::from_arrow_key(bytes[2]);
                    }
                }
                79 => {
                    assert!(bytes[2] >= 80 && bytes[2] <= 83);
                    ke.char = Char::from_fn_key3(bytes[2]);
                }
                _val => unreachable!(
                    "do not know of this byte combination: {:?}, 
                what manner of key did you press, sir?",
                    bytes
                ),
            }
        }
    }

    fn decode_4_bytes(bytes: &[u8], ke: &mut KbdEvent) {
        assert_eq!(bytes.len(), 4);
        // BUG: the below check broke, so it takes to utf84 checks
        // which do not hold true, since it is an esc seq
        // then it crashes with failed assertion
        // if not a valid '4 bytes' utf8 first byte

        if is_utf84(bytes[0], bytes[1], bytes[2], bytes[3]) {
            ke.char = Char::from_utf8(bytes);
        } else {
            assert!(bytes[0] == 27 && bytes[1] == 91 && bytes[3] == 126);
            ke.char = Char::from_cc_extra(bytes[2]);
        }
    }

    // not utf8 anymore
    fn decode_5_bytes(bytes: &[u8], ke: &mut KbdEvent) {
        assert_eq!(bytes.len(), 5);
        assert!(
            bytes[0] == 27 && bytes[1] == 91 && bytes[4] == 126,
            "{bytes:?}"
        );
        assert!(bytes[2] == 50 || bytes[2] == 49);
        ke.char = Char::from_fn_key5(bytes[2], bytes[3]);
    }

    // not utf8
    // 6 bytes < 7 means that there is not a modifiers combination of SUPER + mod(s)
    fn decode_6_bytes(bytes: &[u8], ke: &mut KbdEvent) {
        // escape sequence
        assert_eq!(bytes.len(), 6);
        assert!(bytes[0] == 27 && bytes[1] == 91);
        assert!(bytes[3] == 59);
        assert!([49, 51, 53, 54].contains(&bytes[2]));
        if bytes[2] == 49 {
            ke.modifiers = Modifiers::from_raw67(bytes[4]);
            ke.char = if [65, 66, 67, 68].contains(&bytes[5]) {
                Char::from_arrow_key(bytes[5])
            } else if bytes[5] != 70 && bytes[5] != 72 {
                assert!([80, 81, 82, 83].contains(&bytes[5]), "{bytes:?}");
                Char::from_fn_key3(bytes[5])
            } else {
                assert!(bytes[5] == 70 || bytes[5] == 72);
                if bytes[5] == 70 {
                    Char::CC(CC::End)
                } else {
                    Char::CC(CC::Home)
                }
            }
        } else {
            assert!([51, 53, 54, 49].contains(&bytes[2]));
            ke.modifiers = Modifiers::from_raw67(bytes[4]);
            ke.char = if bytes[2] == 49 {
                match bytes[5] {
                    70 => Char::CC(CC::End),
                    72 => Char::CC(CC::Home),
                    _ => unreachable!("unhandled escape sequence; {:?}", bytes),
                }
            } else {
                assert_eq!(bytes[5], 126);
                Char::from_cc_extra(bytes[2])
            }
        }
    }

    fn decode_7_bytes(bytes: &[u8], ke: &mut KbdEvent) {
        assert_eq!(bytes.len(), 7);
        // escape sequence
        assert!(bytes[0] == 27 && bytes[1] == 91);
        match bytes[6] == 126 && bytes[4] == 59 {
            // fn key + some ctrl, shift, alt mods combination
            true => {
                ke.modifiers = Modifiers::from_raw67(bytes[5]);
                ke.char = Char::from_fn_key5(bytes[2], bytes[3]);
            }
            // arrow keys/extra cc + some super included mods combination
            false => {
                assert_eq!(bytes[4], 49);
                ke.modifiers = Modifiers::from_super7(bytes[5]);
                match bytes[2] {
                    49 if [65, 66, 67, 68].contains(&bytes[6]) => {
                        ke.char = Char::from_arrow_key(bytes[6])
                    }
                    49 if bytes[6] == 70 => ke.char = Char::CC(CC::End),
                    49 if bytes[6] == 72 => ke.char = Char::CC(CC::Home),
                    51 | 53 | 54 if bytes[6] == 126 => ke.char = Char::from_cc_extra(bytes[2]),
                    49 if [80, 81, 82, 83].contains(&bytes[6]) => {
                        ke.char = Char::from_fn_key3(bytes[6])
                    }
                    _ => unreachable!("key combination: {:?} is unhandled", bytes),
                }
            }
        }
    }

    fn decode_8_bytes(bytes: &[u8], ke: &mut KbdEvent) {
        assert_eq!(bytes.len(), 8);
        assert!({
            bytes[0] == 27
                && bytes[1] == 91
                && [49, 50].contains(&bytes[2])
                && bytes[4] == 59
                && bytes[5] == 49
                && bytes[7] == 126
        });
        ke.modifiers = Modifiers::from_super7(bytes[6]);
        ke.char = Char::from_fn_key5(bytes[2], bytes[3]);
    }

    // WARN: design flow
    // should actually get whole strings of bytes and check bytes to correctly decode them into
    // utf8

    // this one is for one input entry
    // need a  function first that split a Ctrl-v received utf8 stream of input into correct chunks
    // of kbdevents
    // INFO: this only works under the premise of receiving 1 input at once
    // basically no ctrl-v or any way to dump multiple chars of input at once
    pub fn decode_ki(bytes: &[u8]) -> KbdEvent {
        let mut ke: KbdEvent = Default::default();
        match bytes.len() {
            1 => decode_1_byte(bytes[0], &mut ke),
            2 => decode_2_bytes(bytes, &mut ke),
            3 => decode_3_bytes(bytes, &mut ke),
            4 => decode_4_bytes(bytes, &mut ke),
            5 => decode_5_bytes(bytes, &mut ke),
            6 => decode_6_bytes(bytes, &mut ke),
            7 => decode_7_bytes(bytes, &mut ke),
            8 => decode_8_bytes(bytes, &mut ke),
            _ => todo!("eh, seriously, a 9 bytes escape sequence? plz stop it already"),
        }

        ke
    }

    // utf8 2 - 3 - 4 bytes values are easy to tell
    // the problem is ascii, it is hard to tell if a value is just an ascii or an escape sequence
    // 2 bytes esc seqs are always 27 then ascii byte
    // escape sequences from 3 to 8 bytes long can always
    // start with 27 then 91|79
    // 3 bytes [27, 79, 80..83] are for f1..4
    // everything else has 91 as a second byte
    // and end with 126|80..83|65..68|70|72
    // 80..83 are for f1..4: 3, 6, 7 bytes
    // 65..68 are for the arrow keys: 3, 6, 7 bytes
    // 70,72 are for the home and end keys: 6, 7 bytes
    // 126 is for everything else: 4, 5, 7, 8 bytes
    //
    // [IMPORTANT]WARN: if we get 3 escape bytes in succession
    // there is no way to tell if they belong to
    // a lone Esc followed by an Alt Esc
    // or an Alt Esc followed by a lone Esc
    // this decoder will consider such bytes to be:
    // a lone Esc followed by an Alt Esc combination
    // INFO: this handles the limitation of decode_ki
    // as this fn can handle a ctrl-v of input
    pub fn decode_ki_kai(bytes: Vec<u8>) -> Vec<Result<KbdEvent, er>> {
        let mut v: Vec<Result<KbdEvent, er>> = vec![];

        let mut bytes = bytes.into_iter();
        while let Some(b0) = bytes.next() {
            match b0 == 27 {
                // this arm covers all escape sequence possibilities
                true => {
                    let b1 = bytes.next();
                    if b1.is_none() {
                        v.push(Ok(KbdEvent {
                            char: Char::CC(CC::ESC),
                            modifiers: Modifiers::from_byte(0),
                        }));

                        return v;
                    }

                    let b1 = b1.unwrap();

                    let b2 = bytes.next();

                    if b2.is_none() {
                        let mut ke = Default::default();
                        decode_2_bytes(&[b0, b1], &mut ke);
                        v.push(Ok(ke));

                        return v;
                    }

                    let b2 = b2.unwrap();

                    match b1 == 91 || b1 == 79 {
                        // surely 3 bytes
                        true if b1 == 79 => {
                            assert!((80..=83).contains(&b2));
                            let mut ke = Default::default();
                            decode_3_bytes(&[b0, b1, b2], &mut ke);

                            v.push(Ok(ke));
                        }
                        // 3 to 8 bytes esc seq
                        true if b1 == 91 => {
                            let mut ke = Default::default();
                            let b3 = bytes.next();
                            if b3.is_none() {
                                decode_3_bytes(&[b0, b1, b2], &mut ke);
                                v.push(Ok(ke));

                                return v;
                            }

                            let b3 = b3.unwrap();
                            match b3 {
                                126 => {
                                    decode_4_bytes(&[b0, b1, b2, b3], &mut ke);
                                    v.push(Ok(ke));
                                }

                                59 => {
                                    let b4 = bytes.next();
                                    let b5 = bytes.next();
                                    if b4.is_none() || b5.is_none() {
                                        v.push(Err(er::other(
                                            "there should have been a 5th and 6th bytes here",
                                        )));
                                        return v;
                                    }

                                    let (b4, b5) = (b4.unwrap(), b5.unwrap());

                                    let b6 = bytes.next();

                                    if b6.is_none() {
                                        decode_6_bytes(&[b0, b1, b2, b3, b4, b5], &mut ke);
                                        v.push(Ok(ke));

                                        return v;
                                    }

                                    let b6 = b6.unwrap();

                                    match b6 {
                                        126 | 80..=83 | 70 | 72 | 65..=68 => {
                                            decode_7_bytes(&[b0, b1, b2, b3, b4, b5, b6], &mut ke);
                                            v.push(Ok(ke));
                                        }
                                        _ => {
                                            decode_6_bytes(&[b0, b1, b2, b3, b4, b5], &mut ke);
                                            bytes = [b6]
                                                .into_iter()
                                                .chain(bytes)
                                                .collect::<Vec<u8>>()
                                                .into_iter();
                                            v.push(Ok(ke));
                                        }
                                    }
                                }
                                _ => {
                                    let b4 = bytes.next();
                                    if b4.is_none() {
                                        v.push(Err(er::other(
                                            "there should have been a 5th and 6th bytes here",
                                        )));
                                        return v;
                                    }
                                    let b4 = b4.unwrap();

                                    match b4 == 126 {
                                        true => {
                                            decode_5_bytes(&[b0, b1, b2, b3, b4], &mut ke);
                                            v.push(Ok(ke));
                                        }
                                        false => {
                                            let (b5, b6) = (bytes.next(), bytes.next());
                                            if b5.is_none() || b6.is_none() {
                                                v.push(Err(er::other(
                                            "there should have been a 5th and 6th bytes here",
                                        )));
                                                return v;
                                            }

                                            let (b5, b6) = (b5.unwrap(), b6.unwrap());
                                            match b6 {
                                                126 => {
                                                    decode_7_bytes(
                                                        &[b0, b1, b2, b3, b4, b5, b6],
                                                        &mut ke,
                                                    );
                                                    v.push(Ok(ke));
                                                }
                                                48..=54 => {
                                                    let b7 = bytes.next();
                                                    if b7.is_none() {
                                                        v.push(Err(er::other(
                                            "there should have been a 5th and 6th bytes here",
                                        )));
                                                        return v;
                                                    }
                                                    let b7 = b7.unwrap();
                                                    decode_8_bytes(
                                                        &[b0, b1, b2, b3, b4, b5, b6, b7],
                                                        &mut ke,
                                                    );
                                                    v.push(Ok(ke));
                                                }
                                                _ => unreachable!("you cant get here:{b6}"),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        false => {
                            v.push(Ok(KbdEvent {
                                char: Char::CC(CC::ESC),
                                modifiers: Modifiers::from_byte(0),
                            }));
                            bytes = [b1, b2]
                                .into_iter()
                                .chain(bytes)
                                .collect::<Vec<u8>>()
                                .into_iter();
                        }
                        _ => unreachable!("im impossible to reach"),
                    }
                }
                // this arm covers all utf8 possibilities
                false => match is_utf81(b0) {
                    true => v.push(Ok({
                        let mut ke = Default::default();
                        decode_1_byte(b0, &mut ke);

                        ke
                    })),
                    false => {
                        let b1 = bytes.next();
                        if b1.is_none() {
                            v.push(Err(er::other("not a valid utf8 nor an esc seq")));

                            return v;
                        }

                        let b1 = b1.unwrap();

                        match is_utf82(b0, b1) {
                            true => {
                                let mut ke = Default::default();
                                decode_2_bytes(&[b0, b1], &mut ke);

                                v.push(Ok(ke));
                            }
                            false => {
                                let b2 = bytes.next();

                                if b2.is_none() {
                                    v.push(Err(er::other("not a valid utf8 nor an esc seq")));

                                    return v;
                                }

                                let b2 = b2.unwrap();

                                match is_utf83(b0, b1, b2) {
                                    true => {
                                        let mut ke = Default::default();
                                        decode_3_bytes(&[b0, b1, b2], &mut ke);

                                        v.push(Ok(ke));
                                    }
                                    // TODO: decode_n_bytes fns should be split on utf8 and esc
                                    // seqs fns
                                    false => {
                                        let b3 = bytes.next();

                                        if b3.is_none() {
                                            v.push(Err(er::other(
                                                "not a valid utf8 nor an esc seq",
                                            )));

                                            return v;
                                        }

                                        let b3 = b3.unwrap();

                                        match is_utf84(b0, b1, b2, b3) {
                                            true => {
                                                let mut ke = Default::default();
                                                decode_4_bytes(&[b0, b1, b2, b3], &mut ke);

                                                v.push(Ok(ke));
                                            }
                                            false => v.push(Err(er::other(
                                                "not a valid utf8 nor an esc seq",
                                            ))),
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }

        v
    }

    // NOTE: the problem with escape sequence is that their length is varying

    fn is_utf81(byte: u8) -> bool {
        byte < 128
    }

    fn is_utf82(byte1: u8, byte2: u8) -> bool {
        (byte1 >= 192 && byte1 < 224) && (byte2 >= 128 && byte2 < 192)
    }

    fn is_utf83(b1: u8, b2: u8, b3: u8) -> bool {
        (b1 >= 224 && b1 < 240) && (b2 >= 128 && b2 < 192) && (b3 >= 128 && b3 < 192)
    }

    fn is_utf84(b1: u8, b2: u8, b3: u8, b4: u8) -> bool {
        (b1 >= 240 && b1 < 248)
            && (b2 >= 128 && b2 < 192)
            && (b3 >= 128 && b3 < 192)
            && (b4 >= 128 && b4 < 192)
    }

    // utf8 string decoder
    // since the module is called utf8_decoder
    // returns error if it doesnt find the expected utf8 bytes
    // crashes if decoding bytes into utf8 str with std::str::from_utf8 returns an error
    pub fn decode_utf8_string(bytes: &[u8]) -> Result<String, std::io::Error> {
        let mut s = String::new();
        let mut bytes = bytes.into_iter();

        while let Some(byte) = bytes.next() {
            // check how many bytes the next utf8 char carries
            match is_utf81(*byte) {
                true => s.push(*byte as char),
                false => {
                    let b2 = bytes.next();
                    let b2 = if b2.is_none() {
                        return Err(std::io::Error::other("not a valid utf8 string"));
                    } else {
                        b2.unwrap()
                    };

                    match is_utf82(*byte, *b2) {
                        true => {
                            s.push_str(std::str::from_utf8(&[*byte, *b2]).expect("not valid utf8"))
                        }
                        false => {
                            let b3 = bytes.next();
                            let b3 = if b3.is_none() {
                                return Err(std::io::Error::other("not a valid utf8 string"));
                            } else {
                                b3.unwrap()
                            };

                            match is_utf83(*byte, *b2, *b3) {
                                true => s.push_str(
                                    std::str::from_utf8(&[*byte, *b2, *b3])
                                        .expect("not valid utf8"),
                                ),
                                false => {
                                    let b4 = bytes.next();
                                    let b4 = if b4.is_none() {
                                        return Err(std::io::Error::other(
                                            "not a valid utf8 string",
                                        ));
                                    } else {
                                        b4.unwrap()
                                    };

                                    assert!(is_utf84(*byte, *b2, *b3, *b4));
                                    s.push_str(
                                        std::str::from_utf8(&[*byte, *b2, *b3, *b4])
                                            .expect("not valid utf8"),
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(s)
    }
}

use utf8_decoder::*;

pub use utf8_decoder::{decode_ki, decode_ki_kai};

// TODO: module needs a bit of refactoring
// particularly decode_ki_kai is a big mess of redundency

pub fn kbd_read() {
    let mut buf: [u8; 8] = [0; 8];
    let mut sol = std::io::stdout().lock();

    let mut input_queue: Vec<u8> = Vec::new();
    let mut si = std::io::stdin().lock();

    // getchar just gets what was pressed
    // it does not detect the press
    // getchar();

    let fps = 60;
    let counter = 1000 / fps;

    while let Ok(n) = si.read(&mut buf) {
        // crossterm's kbd_read (real name event::read()) did not fare any better that my
        // simple stdin reading at detecting the CTRL modifier when used with trappy code points
        // this block inside the loop is the read keyboard key press

        // print!("\r\n{:?}", crossterm::event::read());
        // _ = sol.flush();

        // first we create an empty input stream input_queue
        // then we read from stdin into the buffer buf [u8; 8]
        // then we filter out nil values (untouched)
        // then we push bytes to stream
        // finally stream can decode bytes into proper unicode chars and the user receives them and
        // uses them as they see fit
        //

        // let start = std::time::Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(counter));

        if n > 0 {
            print!("stdout len: {}", n);
            let mut filtered = filter_nil(&buf);
            // input_queue.append(&mut filtered);
            print!(
                "raw_buffer: {:?}\r\nbytes: {:?}\r\ndecoded: {:?}\r\n",
                buf,
                filtered,
                decode_ki(&filtered) // decode_utf8_string(&input_queue) // input_queue
            );

            _ = sol.flush();

            buf = Default::default();
        }
    }
}

use std::io::BufRead;
use std::io::StdinLock;

pub fn read_ki<'a>(reader: &'a mut StdinLock, buffer: &'a mut Vec<u8>) -> &'a mut Vec<u8> {
    buffer.clear();

    let buf = reader.fill_buf().unwrap();
    buffer.extend_from_slice(buf);

    let n = buf.len();
    reader.consume(n);

    buffer
}
