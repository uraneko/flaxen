use crate::events::{Events, EventsConclusion, EventsTrigger};
use crate::kbd_decode::{decode_ki, read_ki, KbdEvent};
use crate::object_tree::{ObjectTree, Term, Zero};
use crate::space_awareness::{Border, Padding};
use std::collections::HashMap;
use std::io::{StdinLock, StdoutLock};
//
// // NOTE: to run an event, all that is needed is that the commissioner knows at the time of the
// // event loop what events need to be run: active event id list
// // how to call those events: access to event call syntax, e.g.,
// // <Term as Events<SomeAnchor>>::fire(term, trigger)
// // themes application and removal should be events based
// // struct EventsQueue<P, T, IE, R>
// // where
// //     IE: Events<P, T>,
// //     T: EventsTrigger,
// //     R: EventsConclusion,
// // (Vec<fn<>>)
//
// // TODO: implement a way to do events and themes
//
// // WARN: this wont work
// // will NOT allow for different Ts or Rs, etc.,
// struct EventsDocument<P, T, IE, R>
// where
//     IE: Events<P, T>,
//     T: EventsTrigger,
//     R: EventsConclusion,
// {
//     id: ID,
//     active: bool,
//     asyncness: bool,
//     f: fn(IE, T) -> R,
//     phantom: std::marker::PhantomData<P>,
// }
//
// impl<'a, P, T, IE, R> EventsDocument<P, T, IE, R>
// where
//     IE: Events<P, T>,
//     T: EventsTrigger,
//     R: EventsConclusion,
// {
//     fn new(asyncness: bool, id: ID, f: fn(IE, T) -> R) -> Self {
//         Self {
//             active: true,
//             asyncness,
//             id,
//             f,
//             phantom: std::marker::PhantomData::<P>,
//         }
//     }
// }
//

// impl this for the object tree
pub trait Commissioner {
    fn render(&self);
}

impl Commissioner for ObjectTree {
    fn render(&self) {}
}

// TODO: change registry to take Permit, Anchor, Trigger and Conclusion strs combinations
// if a combination is there you run the event for that object in the event loop

//
// impl Commissioner {
//     pub async fn process(ke: &KbdEvent) {}
//
//     pub fn clear(writer: &mut StdoutLock) {}
//
//     pub fn render(writer: &mut StdoutLock) {}
// }
//
// struct InnerLogic;
//
// impl Commissioner {
//     fn bind<P, T, R, IE>(events: &mut EventsQueue<P, T, IE, R>, f: fn(IE, T) -> R, eid: ID, id: ID)
//     where
//         // F: Fn(IE, T) -> R,
//         IE: Events<P, T>,
//         T: EventsTrigger,
//         R: EventsConclusion,
//     {
//         let doc = EventsDocument::new(false, eid, f);
//
//         if events.queue.contains_key(&id) {
//             events.queue.get_mut(&id).unwrap().push(doc);
//         } else {
//             events.queue.insert(id, vec![doc]);
//         }
//     }
//
//     fn bind_async<P, T, R, IE>(
//         events: &mut EventsQueue<P, T, IE, R>,
//         f: fn(IE, T) -> R,
//         eid: ID,
//         id: ID,
//     ) where
//         // F: Fn(IE, T) -> R,
//         IE: Events<P, T>,
//         T: EventsTrigger,
//         R: EventsConclusion,
//     {
//         let doc = EventsDocument::new(true, eid, f);
//
//         if events.queue.contains_key(&id) {
//             events.queue.get_mut(&id).unwrap().push(doc);
//         } else {
//             events.queue.insert(id, vec![doc]);
//         }
//     }
//
//     fn release() {}
// }
//
// // TODO:
// // events for
// // space logic,
// // id logic,
// // term, components, input and noneditable logic,
// // have to be implemented by this crate

async fn ragout<P, A, T, IE, R>(
    reader: &mut StdinLock<'static>,
    input: &mut Vec<u8>,
    term: &mut Term,
    writer: &mut StdoutLock<'static>,
) where
    IE: Events<P, A, T>,
    T: EventsTrigger<A>,
    R: EventsConclusion<A>,
{
    let fps = 60;
    let refresh = 1000 / fps;

    loop {
        let input = decode_ki(read_ki(reader, input));

        std::thread::sleep(std::time::Duration::from_millis(refresh));
        // Commissioner::process(&input).await;
        // Commissioner::clear(writer);
        // Commissioner::render(writer);
    }
}

pub enum InitEvent {
    Term(u8),
    Container(&'static [u8], u16, u16, u16, u16),
    Input(&'static [u8], u16, u16, u16, u16),
    NonEdit(&'static [u8], u16, u16, u16, u16, &'static [char]),
}

impl EventsTrigger<CreateObject> for InitEvent {}

pub struct CreateObject;

struct Containerd;

impl EventsConclusion<CreateObject> for () {}

pub struct Anchor;

impl Events<Anchor, CreateObject, InitEvent> for ObjectTree {
    fn fire(&mut self, input: InitEvent) {
        if match input {
            InitEvent::Term(term) => self.term(term),
            InitEvent::Container(id, x0, y0, w, h) => self.container(
                id,
                x0,
                y0,
                w,
                h,
                Border::Uniform('*'),
                Padding::InOut {
                    outer_top: 1,
                    outer_bottom: 1,
                    outer_right: 1,
                    outer_left: 1,
                    inner_top: 1,
                    inner_bottom: 1,
                    inner_right: 1,
                    inner_left: 1,
                },
            ),
            InitEvent::Input(id, x0, y0, w, h) => self.input(
                id,
                x0,
                y0,
                w,
                h,
                Border::Uniform('*'),
                Padding::InOut {
                    outer_top: 1,
                    outer_bottom: 1,
                    outer_right: 1,
                    outer_left: 1,
                    inner_top: 1,
                    inner_bottom: 1,
                    inner_right: 1,
                    inner_left: 1,
                },
            ),
            InitEvent::NonEdit(id, x0, y0, w, h, v) => self.nonedit(
                id,
                x0,
                y0,
                w,
                h,
                v,
                Border::Uniform('*'),
                Padding::InOut {
                    outer_top: 1,
                    outer_bottom: 1,
                    outer_right: 1,
                    outer_left: 1,
                    inner_top: 1,
                    inner_bottom: 1,
                    inner_right: 1,
                    inner_left: 1,
                },
            ),
        }
        .is_err()
        {
            eprintln!("error while pushing new object");
        }
    }
}
