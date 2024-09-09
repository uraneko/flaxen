#![allow(refining_impl_trait)]
use ragout::events::{Events, EventsConclusion, EventsTrigger};
use ragout::input::keyboard::{Char, KbdEvent, Modifiers, CC};
use ragout::object_tree::{Term, Text};

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
struct InnerLogic;

// Permit
struct InputCentral;

impl EventsConclusion<InnerLogic> for Option<String> {}
impl EventsTrigger<InnerLogic> for KbdEvent {}

impl Events<InputCentral, InnerLogic, KbdEvent> for Text {
    fn fire(&mut self, input: KbdEvent) -> Option<String> {
        // input submission
        match (input.char, input.modifiers) {
            // enter hit, submit input from the active input text item
            (Char::CC(CC::CR), Modifiers(0)) => {}
            // just a backspace, erases the char behind the cursor
            (Char::CC(CC::BS), Modifiers(0)) => {}
            // a normal char input with no modifiers
            // put char behind the cursor
            (Char::Char(c), Modifiers(0)) => {}
            // arrow up, move up in input
            (Char::CC(CC::Up), Modifiers(0)) => {}
            // arrow down, move in input
            (Char::CC(CC::Down), Modifiers(0)) => {}
            // arrow right
            (Char::CC(CC::Right), Modifiers(0)) => {}
            // arrow left
            (Char::CC(CC::Left), Modifiers(0)) => {}
            (Char::CC(CC::Home), Modifiers(0)) => {}
            (Char::CC(CC::End), Modifiers(0)) => {}

            _ => return None,
        }

        None
    }
}

fn main() {}

// WARN: BUG: the reason Object types need a Permit Generic is so that an external crate can
// implement events on them
