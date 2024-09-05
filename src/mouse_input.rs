use std::io::{StdoutLock, Write};

use crate::kbd_decode::Modifiers;

// NOTE: theming
// decide which parts of an object will get what style
// then generate the correct ranges every style should be applied on in the whole term buffer that will be rendered
// finally insert the correct styles strings in the term buffer at the time of rendering

#[derive(Default, Debug, Clone)]
pub struct MosEvent {
    gesture: Gesture,
    modifiers: Modifiers,
    position: [u8; 2],
}

#[derive(Default, Debug, Clone)]
pub enum Gesture {
    /// simply moving in any direction
    /// provides the current coordinates of the cursor
    Move(u8, u8),
    /// 1 finger touchpad press
    LeftPress,
    /// release the mouse/touchpad
    Release,
    /// 2 fingers touchpad press
    RightPress,
    /// 3 fingers touchpad press
    WheelePress,
    /// touchpad 2 fingers up
    WheeleUp,
    /// touchpad 2 fingers down
    WheeleDown,
    /// touchpad 2 finger right
    /// can't replicate with the mouse, but it is with the touchpad (probably phone
    /// touch too)
    WheeleRight,
    /// touchpad 2 finger left
    /// don't know if it is possible with the mouse, but it is with the touchpad (probably phone
    /// touch too)
    WheeleLeft,
    #[default]
    None,
}

// TODO: keyboard and mouse input is better sent one key at a time

pub fn decode_mi(bytes: Vec<u8>) -> Vec<MosEvent> {
    let mut rem = bytes.len();
    assert_eq!(rem % 6, 0);

    let mut v: Vec<MosEvent> = Vec::new();

    let mut me = MosEvent::default();
    let mut bytes = bytes.into_iter();

    while rem != 0 {
        let eve = (0..6)
            .into_iter()
            .map(|_| bytes.next())
            .filter(|o| o.is_some())
            .map(|o| o.unwrap())
            .collect::<Vec<u8>>();

        decode_6_bytes(eve, &mut me);

        v.push(me.clone());

        rem -= 6;
    }

    v
}

// th fourth byte; bytes[3] denotes the action being taken as well as the modifiers
// the last 2 bytes are for cursor position (x, y)
// the cursor position returned always start from 33 so should remove 33 from both x and y
// to get the correct position
fn decode_6_bytes(bytes: Vec<u8>, me: &mut MosEvent) {
    // assert mouse escape sequence
    assert_eq!(bytes[0], 27);
    assert_eq!(bytes[1], 91);
    assert_eq!(bytes[2], 77);
    me.modifiers = mouse_modifiers(bytes[3]);
    me.gesture = mouse_gesture(bytes[3], bytes[4], bytes[5]);
    me.position = [bytes[4] - 33, bytes[5] - 33];
}

fn mouse_gesture(byte: u8, bx: u8, by: u8) -> Gesture {
    match byte {
        35 | 43 | 51 | 59 => Gesture::Release,
        67 | 71 | 75 | 83 | 87 | 79 | 91 | 95 => Gesture::None,
        32 | 40 | 48 | 56 => Gesture::LeftPress,
        33 | 41 | 49 | 57 => Gesture::WheelePress,
        34 | 50 | 52 | 48 => Gesture::RightPress,
        96..=123 => match byte % 4 {
            0 => Gesture::WheeleDown,
            1 => Gesture::WheeleUp,
            2 => Gesture::WheeleRight,
            3 => Gesture::WheeleLeft,
            _ => unreachable!("these codes rejuvenate in cycles of 4"),
        },
        _ => unreachable!("all possible cases have been handled"),
    }
}

fn mouse_modifiers(byte: u8) -> Modifiers {
    // shift + 3 finger/wheele click does a paste
    match byte {
        67 | 32..=35 | 96..=99 => Modifiers(0),
        83 | 48..=51 | 112..=115 => Modifiers(2),
        71 | 100..=103 => Modifiers(8),
        75 | 40..=43 | 104..=107 => Modifiers(4),
        87 | 116..=119 => Modifiers(10),          // ctrl + shift
        91 | 56..=58 | 120..=123 => Modifiers(6), // ctrl + alt
        79 | 108..=111 => Modifiers(12),          // shift + alt
        95 | 124..=127 => Modifiers(15),
        _ => unreachable!("all possible cases have been handled"),
    }
}

pub fn enable_mouse_input(writer: &mut StdoutLock) {
    // TODO: the following line enables the terminal to receive mouse events
    _ = writer.write(b"\x1b[?1003h");
}

pub fn disable_mouse_input(writer: &mut StdoutLock) {
    _ = writer.write(b"\x1b[?1003l");
}
