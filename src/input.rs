pub mod keyboard;
pub mod mouse;

// 0 indicates a keyboard input event
// 1 is for mouse
pub fn trace_device(bytes: &[u8]) -> u8 {
    match bytes.len() % 6 == 0 {
        true => match bytes[..3] {
            [27, 91, 77] => 1,
            _ => 0,
        },
        false => 0,
    }
}
