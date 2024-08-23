struct EventsQueue;

use crate::container::Space;
use crate::container::{IDError, IDKind};
use crate::events::{Events, PlaceHolder};
use crate::kbd_decode::{decode_ki, read_ki, KbdEvent};
use crate::Term;
use std::io::{StdinLock, StdoutLock};

pub struct Commissioner {
    events: EventsQueue,
}

impl Commissioner {
    pub fn new() -> Self {
        Self {
            events: EventsQueue,
        }
    }

    pub fn authorize_id(&self, ik: IDKind) -> Result<u8, IDError> {
        match ik {
            IDKind::TextInput => Ok(0), // checks for self last TextId and returns the next one
            IDKind::TextNE => Ok(0),    // checks for self last TextId and returns the next one
            IDKind::Events => Ok(0),
            IDKind::Component => Ok(0),
            IDKind::BufferImage => Err(IDError::ProgramIsUnique),
        }
    }

    pub fn approve_space(&self, space: Space<usize>) -> bool {
        true
    }

    pub async fn process<'a, 'b>(&self, ke: &KbdEvent, term: &mut Term<'a, 'b>) {}

    pub fn clear(&self, writer: &mut StdoutLock) {}

    pub fn render(&self, writer: &mut StdoutLock, term: &mut Term) {}
}

impl<P, T> Commissioner
where
    P: PlaceHolder,
{
    fn bind(&mut self, f: u8, ei: impl Events<P, T>) {}

    fn release() {}
}

async fn ragout<'a, 'b>(
    reader: &mut StdinLock<'static>,
    input: &mut Vec<u8>,
    comr: &mut Commissioner,
    term: &mut Term<'a, 'b>,
    writer: &mut StdoutLock<'static>,
) {
    let fps = 60;
    let refresh = 1000 / fps;

    loop {
        let input = decode_ki(read_ki(reader, input));

        std::thread::sleep(std::time::Duration::from_millis(refresh));
        comr.process(&input, term).await;
        comr.clear(writer);
        comr.render(writer, term);
    }
}
