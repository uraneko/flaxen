use ragout::kbd_decode::read_ki;
use ragout::mouse_input::*;
use ragout::raw_mode::raw_mode;
use ragout::winsize;

fn main() {
    print!("{:?}", winsize::from_ioctl());

    _ = raw_mode();
    let mut writer = std::io::stdout().lock();
    //need to call kbd decode's read_ki
    enable_mouse_input(&mut writer);

    let mut reader = std::io::stdin().lock();

    let mut inp = Vec::new();

    loop {
        _ = read_ki(&mut reader, &mut inp);
        // receive mouse input and do stuff with it
        print!("{:?}\r\n", inp);

        if !inp.is_empty() && inp[0] == 3 {
            break;
        }

        let e = decode_mi(inp.drain(..).collect());

        println!("{:?}\r\n", e);
    }

    disable_mouse_input(&mut writer);
}
