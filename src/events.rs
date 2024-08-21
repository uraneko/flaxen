// NOTE: user can create their own type and implement Event for it
// then they can make event queues for that event implementor
trait Event {}

#[derive(Debug)]
struct EventQueue<T>
where
    T: Event,
{
    events: VecDeque<T>,
}

impl<T> EventQueue<T>
where
    T: Event,
{
    fn push(&mut self, item: T) {
        self.events.push_back(item);
    }

    fn pop(&mut self) -> T {
        self.events.pop_back().unwrap()
    }

    fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }
}

use std::collections::VecDeque;

#[derive(Debug)]
enum InputEvent {
    A,
    B,
    C,
}

impl Event for InputEvent {}

// responsible for managing event queues and running events
struct Commissioner;

impl Commissioner {
    // start watching for events
    fn start(&self) {}

    // add events for observation
    fn extend(&self) {}

    // dont observe the event with the given id
    fn release(&self) {}

    // restart observing the event with the given id
    fn restore(&self) {}
}

impl Drop for Commissioner {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_events() {
        println!("0");
        let mut vd: EventQueue<InputEvent> = EventQueue::new();
        println!("1");
        vd.push(InputEvent::A);

        println!("3");
        println!("{:?}", vd);
    }
}
