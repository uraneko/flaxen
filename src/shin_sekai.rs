enum TItem {
    Terminal,
    Input,
    Prompt,
    Popup,
}

trait Container {
    // save all saved settings - sgr, cursor positon and char set -
    fn _7();

    // restore params saved in _7()
    fn _8();

    // move cursor to position (x, y)
    fn f();

    // line deletion
    fn k();

    // screen deletion,
    // NOTE: only implement for terminal struct
    fn j();

    // reset to starting initial state
    fn c();

    // clear selected parameter, resetting them
    fn l();

    fn write();

    fn active();

    fn screenshot();
}

// input trait is anything that the end user can edit directly
trait Input: Container + InputBehavior {
    // cursor up
    fn A();

    // cursor down
    fn B();

    // cursor right
    fn C();

    // cursor left
    fn D();

    // move cursor to Home position
    fn H();

    // Erase all characters from the active position to the end of the current line.
    // The active position is not changed.
    fn K();

    // save cursor position same as _7
    fn s();

    // restore cursor position, same as _8
    fn u();
}

trait InputBehavior {
    fn put_char();
    fn del_char();

    fn new_line();

    fn to_end();
    fn to_home();

    fn to_right_1();
    fn to_left_1();

    fn to_right_word();
    fn to_left_word();

    fn del_right_word();
    fn del_left_word();

    fn del_line();

    fn relocate();
    fn edit();
    fn submit();
}

trait Stylize {}

trait Styled: Stylize + Container {
    // select graphic rendition
    fn m();
}

trait Render: Container {
    // render this item on terminal
    fn render();

    // clear this item from the terminal display
    fn clear();
}

// WARN: these are not keyboard events, these are program events
trait Events {
    // start watching for events
    fn observe();

    // add events for observation
    fn extend();

    // dont observe the event with the given id
    fn release();

    // restart observing the event with the given id
    fn restore();
}
