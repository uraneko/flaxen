use crate::events::{Events, EventsConclusion, EventsTrigger};
use crate::input::keyboard::{Char, KbdEvent, Modifiers, CC};
use crate::object_tree::{Term, Text};
use crate::raw_mode::termios;

pub struct WindowResized(u16, u16);

impl WindowResized {
    pub fn new(w: u16, h: u16) -> Self {
        Self(w, h)
    }
}

impl EventsTrigger<CoreEvents> for WindowResized {}

impl EventsConclusion<CoreEvents> for () {}

// Anchor
pub struct CoreEvents;

// Permit
pub struct TermCentral;

// BUG: this is broken
// it does nothing
impl Events<TermCentral, CoreEvents, WindowResized> for Term {
    fn fire(&mut self, values: WindowResized) {
        self.containers.iter_mut().for_each(|c| {
            c.items.iter_mut().for_each(|t| {
                t.rescale(values.0, values.1);
            });
            c.rescale(values.0, values.1);
        });
        self.rescale(values.0, values.1);
    }
}

// Anchor
pub struct InnerLogic;

// Permit
pub struct BasicInput;

impl EventsConclusion<InnerLogic> for Vec<Option<char>> {}
impl EventsTrigger<InnerLogic> for (&KbdEvent, &[Vec<Option<char>>]) {}

use std::io::StdoutLock;

impl<'a> Events<BasicInput, InnerLogic, (&'a KbdEvent, &'a [Vec<Option<char>>])> for Text {
    fn fire(&mut self, values: (&'a KbdEvent, &'a [Vec<Option<char>>])) -> Vec<Option<char>> {
        let (input, cache) = values;
        // input submission
        match (&input.char, &input.modifiers) {
            // enter hit, submit input from the active input text item
            (Char::CC(CC::CR), Modifiers(0)) => return self.submit(),
            // just a backspace, erases the char behind the cursor
            (Char::CC(CC::BS), Modifiers(0)) => self.delete(),
            // a normal char input with no modifiers
            // put char behind the cursor
            (Char::Char(c), Modifiers(0)) => {
                self.put_char(*c);
            }
            // arrow up, move up in input
            (Char::CC(CC::Up), Modifiers(0)) => self.up(),
            // arrow down, move in input
            (Char::CC(CC::Down), Modifiers(0)) => self.down(),

            // arrow right
            (Char::CC(CC::Right), Modifiers(0)) => self.right(),
            // arrow left
            (Char::CC(CC::Left), Modifiers(0)) => self.left(),
            // go to home
            (Char::CC(CC::Home), Modifiers(0)) => self.home(),
            // go to end
            (Char::CC(CC::End), Modifiers(0)) => self.end(),
            // go to home vertical
            (Char::CC(CC::Home), Modifiers(4)) => self.homev(),
            // go to end
            (Char::CC(CC::End), Modifiers(4)) => self.endv(),

            (Char::CC(CC::Up), Modifiers(4)) => self.history_up(cache),

            (Char::CC(CC::Down), Modifiers(4)) => self.history_down(cache),

            _ => return vec![],
        }

        self.change = 2;

        vec![]
    }
}

pub struct InteractiveSwitch;
pub struct Interactive;

impl EventsTrigger<Interactive> for (&KbdEvent, &mut StdoutLock<'static>) {}
impl EventsConclusion<Interactive> for () {}

impl<'a> Events<InteractiveSwitch, Interactive, (&'a KbdEvent, &'a mut StdoutLock<'static>)>
    for Term
{
    fn fire(&mut self, inputs: (&'a KbdEvent, &'a mut StdoutLock<'static>)) {
        let (input, writer) = (inputs.0, inputs.1);
        let id = if input.char == Char::CC(CC::TAB) {
            if input.modifiers == Modifiers(0) {
                self.interactable_next()
            } else if input.modifiers == Modifiers(8) {
                self.interactable_prev()
            } else {
                return;
            }
        } else {
            return;
        };

        if id.is_some() {
            self.make_active(&id.unwrap());
        }
    }
}

impl EventsTrigger<CoreEvents> for (&KbdEvent, &mut StdoutLock<'static>, &termios) {}

impl<'a> Events<BasicInput, CoreEvents, (&'a KbdEvent, &'a mut StdoutLock<'static>, &'a termios)>
    for Term
{
    fn fire(&mut self, values: (&'a KbdEvent, &'a mut StdoutLock<'static>, &'a termios)) {
        let (input, writer, ts) = (values.0, values.1, values.2);
        match (&input.char, &input.modifiers) {
            (Char::Char('c'), Modifiers(2)) => {
                // save all input objects cache
                self.cache
                    .iter()
                    .for_each(|(k, _)| self.save_input(k, None));

                // go back to cooked mode using the cached termios instance
                crate::raw_mode::cooked_mode(&ts);
                // exit alternate screen back to the default terminal screen
                crate::leave_alternate_screen(writer);

                // exit the program
                std::process::exit(0);
            }
            (Char::Char('l'), Modifiers(2)) => {
                // clear terminal display
                self.clear(writer);
                // render terminal buffer
                self.render(writer)
            }
            // other key events are irrelevant to this Events implementation
            _ => {}
        }
    }
}
