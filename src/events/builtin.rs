use crate::events::{Events, EventsConclusion, EventsTrigger};
use crate::kbd_decode::{Char, KbdEvent, Modifiers, CC};
use crate::object_tree::{Term, Text};
use crate::space::SpaceAwareness;

struct WindowResized(u16, u16);

impl EventsTrigger<CoreEvents> for WindowResized {}

impl EventsConclusion<CoreEvents> for () {}

// Anchor
struct CoreEvents;

// Permit
struct TermCentral;

impl Events<TermCentral, CoreEvents, WindowResized> for Term {
    fn fire(&mut self, input: WindowResized) {
        self.containers.iter_mut().for_each(|c| {
            c.items.iter_mut().for_each(|t| {
                t.rescale(input.0, input.1);
            });
            c.rescale(input.0, input.1);
        });
        self.rescale(input.0, input.1);
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

impl EventsTrigger<Interactive> for &KbdEvent {}
impl EventsConclusion<Interactive> for Option<[u8; 3]> {}

impl<'a> Events<InteractiveSwitch, Interactive, &'a KbdEvent> for Term {
    fn fire(&mut self, input: &'a KbdEvent) -> Option<[u8; 3]> {
        if input.char == Char::CC(CC::TAB) {
            if input.modifiers == Modifiers(0) {
                return Some([0, 1, 0]);
            } else if input.modifiers == Modifiers(8) {
                return Some([0, 0, 0]);
            }
        }

        None
    }
}

impl Text {}
