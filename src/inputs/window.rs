use crate::console::winsize::winsize;

#[derive(Debug)]
pub enum WindowEvent {
    WindowResized,
    // WindowGainedFocus,
    // WindowLostFocus,
    // WindowMaximized,
    // WindowMinimized,
    // WindowClosed,
    // WindowFullscreened,
    // WindowWindowed,
}

pub struct WindowResized;

use crate::components::Term;

// resolve linux display/compositor protocol
// returns 0 for wayland or 1 for x11
#[cfg(target_os = "linux")]
fn compositor() -> u8 {
    let is_wayland = std::process::Command::new("printenv")
        .arg("WAYLAND_DISPLAY")
        .output();
    // if err then no wayland
    if is_wayland.is_err() {
        1
    } else {
        0
    }
}

fn window_resized() /* -> WindowResized */
{
    #[cfg(target_os = "linux")]
    match compositor() {
        0 => {
            // wayland
        }
        1 => {
            // x11
        }
        _ => unreachable!(),
    }

    #[cfg(target_os = "windows")]
    // windows get window resize values
    #[cfg(target_os = "macos")]
    // macos get window resize values
    if cfg!(x86_64) {
        // amd64 macos
    } else if cfg!(aarch64) {
        // apple silicon aarch64 macos
    }
}
