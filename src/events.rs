// NOTE: user can create their own type and implement Event for it
// then they can make event queues for that event implementor

use std::collections::VecDeque;

use crate::container::Commissioner;

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

use crate::container::EventId;

trait Events<T>: HasId {
    const ID: u8; // the id of the instance of whatever the trait is being implemented on
                  // this assures that the generic trait is only implemented on 1 instance of the type
                  //
    /// takes a kbd input event and returns an event conclusion
    /// this functions body always has a match statement that matches on the key and modifiers of
    /// the input and runs the matching function implemented by the time this trait is implemented
    /// on
    /// type T is so that the trait can be implemented for the same type many times
    /// the default way of doing this is that for every impl of Events for the same type, you  creating an empty struct type and use as the generic of that particular impl
    fn fire(&self, input: KbdEvent) -> impl EventConclusion;
    fn id(&self) -> EventId;
    fn validate(&self);
}

use crate::container::Edges;

trait HasId {
    fn id(&self) -> u8;
}
