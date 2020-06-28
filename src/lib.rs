use crate::typemap::TypeMap;
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;

pub mod typemap;

#[derive(Default)]
pub struct Injector<'a> {
    values: TypeMap,
    parent: Option<&'a mut Injector<'a>>,
}

impl<'a> Injector<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parent(self, parent: &'a mut Injector<'a>) -> Self {
        Self {
            values: self.values,
            parent: Some(parent),
        }
    }

    pub fn add_value<T: 'static>(&mut self, v: T) {
        self.values.insert(v);
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        if let Some(v) = self.values.get::<T>() {
            Some(v)
        } else if let Some(p) = &self.parent {
            p.get::<T>()
        } else {
            None
        }
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if let Some(v) = self.values.get_mut::<T>() {
            Some(v)
        } else if let Some(p) = &mut self.parent {
            p.get_mut::<T>()
        } else {
            None
        }
    }

    pub fn call<A, F: CallInjector<A, R>, R>(&self, f: F) -> R {
        CallInjector::call(&f, self)
    }
}

/// Handler converter factory
/// Async handler converter factory
pub trait CallInjector<Args, Res>: Clone {
    fn call(&self, i: &Injector) -> Res;
}

pub trait AsyncCallInjector<Args, Res, Output>: Clone + 'static
where
    Res: Future<Output = Output>,
{
    fn call(&self, i: &Injector) -> Res;
}

impl<F, Res> CallInjector<(), Res> for F
where
    F: Fn() -> Res + Clone,
{
    fn call(&self, _: &Injector) -> Res {
        (self)()
    }
}

impl<F, Res, Output> AsyncCallInjector<(), Res, Output> for F
where
    Res: Future<Output = Output>,
    F: Fn() -> Res + Clone + 'static,
{
    fn call(&self, _: &Injector) -> Res {
        (self)()
    }
}

macro_rules! factory_tuple ({ $(($n:tt, $T:ident)),+} => {
        impl<Func, $($T: 'static,)+ Res> CallInjector<($($T,)+), Res> for Func
        where Func: Fn($(&$T,)+) -> Res + Clone,
        {
            fn call(&self, inj: &Injector) -> Res {
                (self)($(inj.get::<$T>().unwrap(),)+)
            }
        }

        // impl<Func, $($T,)+ Res, Output> AsyncCallInjector<($($T,)+), Res, Output> for Func
        // where Func: Fn($($T,)+) -> Res + Clone + 'static,
        //       Res: Future<Output = Output>,
        // {
        //     fn call(&self, param: ($($T,)+)) -> Res {
        //         (self)($(param.$n,)+)
        //     }
        // }
    });

#[rustfmt::skip]
mod m {
    use super::*;

    factory_tuple!((0, A));
    factory_tuple!((0, A), (1, B));
    factory_tuple!((0, A), (1, B), (2, C));
    factory_tuple!((0, A), (1, B), (2, C), (3, D));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H), (8, I));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H), (8, I), (9, J));
}

#[cfg(test)]
mod tests {
    use crate::Injector;
    use std::sync::{Arc, Mutex};

    #[test]
    fn it_works() {
        fn plus_one(a: &u32) -> u32 {
            a + 1
        }

        let mut i = Injector::default();
        i.add_value(10u32);

        assert_eq!(i.call(plus_one), 11);
    }

    #[test]
    fn it_works_with_non_copy() {
        fn plus_one(a: &Arc<Mutex<u32>>) -> u32 {
            *a.lock().unwrap() + 1
        }
        fn inc(a: &Arc<Mutex<u32>>) -> () {
            *a.lock().unwrap() += 1;
        }

        let mut i = Injector::default();
        i.add_value(Arc::new(Mutex::new(20u32)));

        assert_eq!(i.call(plus_one), 21);
        i.call(inc);
        assert_eq!(i.call(plus_one), 22);
    }

    #[test]
    fn it_works_with_multiple_values() {
        fn plus(a: &u32, b: &u8) -> u32 {
            a + (*b as u32)
        }
        fn plus_3(a: &u32, b: &u8, c: &u16) -> u32 {
            a + (*b as u32) + (*c as u32)
        }

        let mut i = Injector::default();
        i.add_value(5u32);
        i.add_value(80u8);
        i.add_value(100u16);

        assert_eq!(i.call(plus), 85);
        assert_eq!(i.call(plus_3), 185);
    }

    #[test]
    fn it_works_with_inheritance() {
        fn plus(a: &u32, b: &u8) -> u32 {
            a + (*b as u32)
        }
        fn plus_3(a: &u32, b: &u8, c: &u16) -> u32 {
            a + (*b as u32) + (*c as u32)
        }

        let mut i1 = Injector::default();
        i1.add_value(5u32);
        let mut i2 = Injector::default().with_parent(&mut i1);
        i1.add_value(5u32);

        assert_eq!(i.call(plus), 85);
        assert_eq!(i.call(plus_3), 185);
    }
}
