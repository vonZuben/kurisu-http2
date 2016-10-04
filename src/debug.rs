//! At the time of creating this, using debug_assertions is experimental
//! on blocks of code within functions. This allows me to get the debug_assertions
//! behaviour on any arbitrary expressions.

use std::fmt::Debug;

/// A macro to wrap around arbitrary expressions
/// that should only be run in debug builds
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! drun {
    ( $run:expr ) => {
        {
            println!("=================");
            $run
            println!("=================");
        }
    };
}

/// No-op version of the macro for release builds
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! drun {
    ( $run:expr ) => { };
}

#[cfg(test)]
mod debug_print_tests {
    #[test]
    fn drun_test() {
        let mut a = 0i32;

        drun!({ println!("drun"); a = 5i32; });

        debug_assert_eq!(a, 5);

        a = 10i32;

        assert_eq!(a, 10);
    }
}
