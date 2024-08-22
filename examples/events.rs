fn main() {
    let a = A::<'a'>::new(8);
    let b = A::<'b'>::new(9);

    <A<'a'> as G<InnerLogic>>::a(&a); // doesnt error

    // <A<'b'> as G<InnerLogic>>::a(&b); // errors

    let c = A::<'a'>::new(34);

    <A<'a'> as G<InnerLogic>>::a(&c); // doesnt error
}

struct InnerLogic;

struct A<const ID: char> {
    id: u8,
}

impl<const ID: char> A<ID> {
    const fn id(&self) -> u8 {
        self.id
    }

    fn new(id: u8) -> Self {
        Self { id }
    }
}

trait G<T> {
    const IDMatch: char;

    fn a(&self);
}

impl<InnerLogic> G<InnerLogic> for A<'a'> {
    const IDMatch: char = 'a';

    fn a(&self) {
        println!("from G<InnerLogic>: a(): {}", self.id);
    }
}
