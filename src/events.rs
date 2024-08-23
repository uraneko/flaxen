// NOTE: user can create their own type and implement Event for it
// then they can make event queues for that event implementor

use std::collections::VecDeque;

use crate::commissioner::Commissioner;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_events() {
        // println!("0");
        // let mut vd: EventQueue<InputEvent> = EventQueue::new();
        // println!("1");
        // vd.push(InputEvent::A);
        //
        // println!("3");
        // println!("{:?}", vd);
    }
}

use crate::KbdEvent;

trait EventConclusion {}

enum EventId {}
enum Edges {}

pub trait PlaceHolder {}

// P is a PlaceHolder type that allows any crate to implement this trait for types native to
// this crate
// T is the trigger type which will be matched on to trigger an event
pub trait Events<P, T>: HasId
where
    P: PlaceHolder,
{
    // the id of the instance of whatever the trait is being implemented on
    // this assures that the generic trait is only implemented on 1 instance of the type
    const ID: u8;

    /// takes a kbd input event and returns an event conclusion
    /// this functions body always has a match statement that matches on the key and modifiers of
    /// the input and runs the matching function implemented by the time this trait is implemented
    /// on
    /// type T is so that the trait can be implemented for the same type many times
    /// the default way of doing this is that for every impl of Events for the same type, you  creating an empty struct type and use as the generic of that particular impl
    fn fire(&self, input: T) -> impl EventConclusion;
    fn id(&self) -> EventId;
    fn validate(&self);
}

trait HasId {
    fn id(&self) -> u8;
}
