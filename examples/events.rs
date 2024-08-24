use std::ops::Deref;

fn main() {
    let a = A::<'a'>::new(8);
    let b = A::<'b'>::new(9);

    <A<'a'> as G<InnerLogic>>::a(&a); // doesnt error

    // <A<'b'> as G<InnerLogic>>::a(&b); // errors

    let c = A::<'a'>::new(34);

    <A<'a'> as G<InnerLogic>>::a(&c); // doesnt error

    <A<'b'> as G<OverReach>>::a(&b);

    let y = Y {
        v: vec![Box::new(A::<'a'>::new(87)), Box::new(A::<'2'>::new(89))],
    };

    println!("{:#?}", y);

    y.v[0].as_a().a();
}

#[derive(Debug)]
struct Y<const ID: char> {
    v: Vec<Box<dyn AllA<ID>>>,
}

// 2 ways this could be implemented
// 1:
// - make the vec take boxes of dyn impl TraitThatAll'Type<CONST>'VariationsImplement
// - implement methods as_type and as_mut_type from the Boxed implementor
// that way we can store and manipulate the typeS instances
// 2:
// - remove the associated constant making the type variation types effectively one type
// - this makes it impossible to have different impl SpecialTrait<Generic> for Type<CONSTANT>,
// that is we cant use the same Anchor generic for different constants variations of the type
// consequently all impl SpecialTrait<Generic> for Type impls will be exposed to all type instances
// which means we'll have to use Generics extensively
//

// again this doesnt work
// you end up providin ga value for id for this to work
// which completely misses the point
trait AllA<const ID: char>: std::fmt::Debug {
    fn as_a(&self) -> &A<ID> {
        self.as_a()
    }

    fn as_mut_a(&mut self) -> &mut A<ID> {
        self.as_mut_a()
    }
}

// TODO: use PhantomData instead of associated const

// fn from_cst0(&mut self, &A<'0'>) -> Self {
//     A::<'0'> {
//         ..self
//     }
// }

// NOTE: maybe this can have a convergence type with a predecated associated const value that all
// types converge back to

impl<const ID: char> AllA<ID> for A<ID> {}

struct InnerLogic;

#[derive(Debug)]
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
    fn a(&self);
}

impl G<InnerLogic> for A<'a'> {
    fn a(&self) {
        println!("from G<InnerLogic>: a(): {}", self.id);
    }
}

struct OverReach;

impl<const ID: char> G<OverReach> for A<ID> {
    fn a(&self) {
        println!("this is overreach being implemented for all A<ID> types here");
    }
}
