use std::ops::Deref;

fn main() {
    let mut a = A::new(8);
    a.permit::<D>();

    let mut b = A::new(9);
    b.permit::<B>();

    println!("{:?}", a.id());

    s::<Zero>();

    <A as G<D, InnerLogic>>::a(&a); // doesnt error

    // <A<O> as G<InnerLogic>>::a(&b.morph::<O>()); // errors

    let mut c: A = A::new(34);
    c.permit::<D>();

    <A as G<D, InnerLogic>>::a(&c); // doesnt error

    <A as G<B, OverReach>>::a(&b);

    let y: Y = Y {
        v: vec![A::new(87), A::new(89)],
    };

    println!("{:#?}", y);

    <A as G<B, OverReach>>::a(&y.v[0]);
    <A as G<D, OverReach>>::a(&y.v[0]);
}

#[derive(Debug)]
struct Y {
    v: Vec<A>,
}

#[derive(Debug)]
struct Zero;
#[derive(Debug)]
struct D;
#[derive(Debug)]
struct B;
#[derive(Debug)]
struct O;

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

struct InnerLogic;

#[derive(Debug)]
struct A {
    id: u8,
    registry: Vec<&'static str>,
}

// TODO: need a way to constrict A<Anchor> divergence types

fn s<A>() {
    println!("{}", std::any::type_name::<A>());
}

// NOTE: Zero here is a placeholder, it is not an actual type, nor the actual type Zero
impl A {
    const fn id(&self) -> u8 {
        self.id
    }

    fn new(id: u8) -> A {
        A {
            id,
            registry: vec![],
        }
    }

    fn permit<P>(&mut self) {
        self.registry.push(std::any::type_name::<P>());
    }

    // TODO: should i consume or just take by ref and use temporarily
    // TODO: do both, make a morph_ref, a morph_val,
    // NOTE: this does not make a difference in the return type, that will always be an A<Anchor>
    // owned value,
    // the reason there is not a morph_mut_ref is the same, because we only return an owned value
    // fn morph<M>(&self) -> A<M> {
    //     assert!(self.allowed.contains(&std::any::type_name::<M>()));
    //     A::<M> {
    //         phantom: std::marker::PhantomData::<M>,
    //         id: self.id,
    //         allowed: self.allowed.clone(),
    //     }
    // }
    //
    // fn morph_sib<'a, M>(&self, a: &'a mut A<M>) -> &'a mut A<M> {
    //     assert!(self.allowed.contains(&std::any::type_name::<M>()));
    //     *a = A::<M> {
    //         phantom: std::marker::PhantomData::<M>,
    //         id: self.id,
    //         allowed: self.allowed.clone(),
    //     };
    //
    //     a
    // }
    //
    // fn allow<Y>(&mut self) {
    //     self.allowed.push(std::any::type_name::<Y>());
    // }
    //
    // fn converge(&self) -> A<Zero> {
    //     A::<Zero> {
    //         phantom: std::marker::PhantomData::<Zero>,
    //         id: self.id,
    //         allowed: vec![],
    //     }
    // }
}

trait G<P, T> {
    fn a(&self);
}

impl G<D, InnerLogic> for A {
    fn a(&self) {
        println!("from G<D, InnerLogic>: a(): {}", self.id);
    }
}

struct OverReach;

// NOTE: Anchor is a placeholder for a real type name
impl<Anchor> G<Anchor, OverReach> for A {
    fn a(&self) {
        println!("this is overreach being implemented for all Anchor types here");
    }
}
