#![allow(dead_code)]

mod dp {
    use std::fmt::Debug;

    #[cfg(debug)]
    #[inline(always)]
    fn debug_print<T: Debug>(item: T) {
        println!("{:?}", item);
    }

    #[cfg(not(debug))]
    #[inline(always)]
    fn debug_print<T: Debug>(_: T) {
        //noop!();
        println!("hello");
    }
}


macro_rules! debug {
    ($d:ident) => (
        //::dp::debug_print($d);
        unimplemented!();
    )
}

