use std::any::Any;
use std::marker::PhantomData;

fn main() {
    // let v: Vec<u8> = vec![1, 2];

    let d = vec![
        <S<Z> as Events<A1, u8>>::fire as *const (),
        <S<Y> as Events<A2, i16>>::fire as *const (),
    ];

    let mut s1 = S {
        s: 1,
        phan: PhantomData::<Z>,
    };
    let mut s2 = S {
        s: 2,
        phan: PhantomData::<Y>,
    };

    unsafe {
        // let pointer = <S<Z> as Events<A1, u8>>::fire as *const ();
        let function = unsafe { std::mem::transmute::<*const (), fn(&mut S<Z>, u8)>(d[0]) };
        function(&mut s1, 23);
    }

    println!(
        "const ptr: {:p}\r\nconst ptr: {:p}\r\n{}",
        <S<Z> as Events<A1, u8>>::fire as *const fn(&mut S<Z>, u8),
        d[0] as *const fn(&mut S<Z>, u8),
        d[0] as *const fn(&mut S<Z>, u8)
            == <S<Z> as Events<A1, u8>>::fire as *const fn(&mut S<Z>, u8),
    );
    // v[1].downcast_ref::<&for<'a> fn(&'a mut S<Y>, i16)>()
    //     .expect("hah2?")(&mut s2, -64);

    // <S<Z> as Events<A1, u8>>::fire(&mut s1, 23);
    // <S<Y> as Events<A2, i16>>::fire(&mut s2, -31);
    //
    // TODO: how to solve
    // need the correct fn signature
    //
}

trait Conclusion {}
trait Trigger {}

trait Events<Anchor, T>
where
    T: Trigger,
{
    fn fire(&mut self, value: T) -> impl Conclusion;
}

#[derive(Debug)]
struct S<T> {
    s: u8,
    phan: PhantomData<T>,
}

impl Trigger for u8 {}
impl Trigger for i16 {}
impl Conclusion for () {}

#[derive(Debug)]
struct Z;
#[derive(Debug)]
struct Y;
#[derive(Debug)]
struct A1;
#[derive(Debug)]
struct A2;

impl Events<A1, u8> for S<Z> {
    fn fire(&mut self, value: u8) -> impl Conclusion {
        self.s += value;
        println!("events<A1, u8, Z>: {:?}", self);
    }
}

impl Events<A2, i16> for S<Y> {
    fn fire(&mut self, value: i16) -> impl Conclusion {
        self.s = (self.s as i16 - value) as u8;
        println!("events<A1, i16, Y>: {:?}", self);
    }
}
