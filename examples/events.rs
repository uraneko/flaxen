use std::ops::Deref;

fn main() {
    let mut a = A::zero(8);
    a.allow::<D>();

    let mut b = A::zero(9);
    b.allow::<B>();

    println!("{:?}", a.id());

    s::<Zero>();

    <A<D> as G<InnerLogic>>::a(&a.morph::<D>()); // doesnt error

    // <A<O> as G<InnerLogic>>::a(&b.morph::<O>()); // errors

    let mut c: A<B> = A::<B>::new(34);
    c.allow::<D>();

    <A<D> as G<InnerLogic>>::a(&c.morph::<D>()); // doesnt error

    <A<B> as G<OverReach>>::a(&b.morph::<B>());

    let y: Y<Zero> = Y {
        v: vec![A::<Zero>::new(87), A::<Zero>::new(89)],
    };

    println!("{:#?}", y);

    y.v[0].a();
}

#[derive(Debug)]
struct Y<Anchor> {
    v: Vec<A<Anchor>>,
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

// TODO: use PhantomData instead of associated const

// NOTE: maybe this can have a convergence type with a predecated associated const value that all
// types converge back to

struct InnerLogic;

#[derive(Debug)]
struct A<Anchor> {
    id: u8,
    phantom: std::marker::PhantomData<Anchor>,
    allowed: Vec<&'static str>,
}

// TODO: need a way to constrict A<Anchor> divergence types

impl A<Zero> {
    fn zero(id: u8) -> A<Zero> {
        A::<Zero> {
            id,
            phantom: std::marker::PhantomData::<Zero>,
            allowed: vec![],
        }
    }
}

fn s<A>() {
    println!("{}", std::any::type_name::<A>());
}

// NOTE: Zero here is a placeholder, it is not an actual type, nor the actual type Zero
impl<T> A<T> {
    const fn id(&self) -> u8 {
        self.id
    }

    fn new<U>(id: u8) -> A<U> {
        A::<U> {
            id,
            phantom: std::marker::PhantomData::<U>,
            allowed: vec![],
        }
    }

    // TODO: should i consume or just take by ref and use temporarily
    // TODO: do both, make a morph_ref, a morph_val,
    // NOTE: this does not make a difference in the return type, that will always be an A<Anchor>
    // owned value,
    // the reason there is not a morph_mut_ref is the same, because we only return an owned value
    fn morph<M>(&self) -> A<M> {
        assert!(self.allowed.contains(&std::any::type_name::<M>()));
        A::<M> {
            phantom: std::marker::PhantomData::<M>,
            id: self.id,
            allowed: self.allowed.clone(),
        }
    }

    fn morph_sib<'a, M>(&self, a: &'a mut A<M>) -> &'a mut A<M> {
        assert!(self.allowed.contains(&std::any::type_name::<M>()));
        *a = A::<M> {
            phantom: std::marker::PhantomData::<M>,
            id: self.id,
            allowed: self.allowed.clone(),
        };

        a
    }

    fn allow<Y>(&mut self) {
        self.allowed.push(std::any::type_name::<Y>());
    }

    fn converge(&self) -> A<Zero> {
        A::<Zero> {
            phantom: std::marker::PhantomData::<Zero>,
            id: self.id,
            allowed: vec![],
        }
    }
}

trait G<T> {
    fn a(&self);
}

impl G<InnerLogic> for A<D> {
    fn a(&self) {
        println!("from G<InnerLogic>: a(): {}", self.id);
    }
}

struct OverReach;

// NOTE: Anchor is a placeholder for a real type name
impl<Anchor> G<OverReach> for A<Anchor> {
    fn a(&self) {
        println!("this is overreach being implemented for all A<Anchor> types here");
    }
}
