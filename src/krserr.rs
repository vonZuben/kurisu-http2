/// Module for this crate to have a uniform way of
/// creating and passing around errors that can carry
/// dynamic information about errors in a chain and
/// write to a log.
///
/// TODO
/// carry type info in error for dealing with specific errors cases
/// in different ways
///
/// NOTE
/// - This creates a linked list of errors. Uses dynamic dispatch for
/// writing to any buffer (eg. A log).
/// - The chain is meant to be useful to for adding error related information
/// that can be propagated up though functions until a point where you want to
/// deal with the specific error case

/// A link in the chain of errors (Forms a linked list)
#[derive(Debug)]
pub struct ErrLink {
    error: Box<::std::error::Error>,
    link: Option<Box<ErrLink>>,
}

impl ErrLink {
    fn attach_links<E>(self, err: E) -> ErrLink where E: ::std::error::Error + 'static {
        ErrLink {
            error: err.into(),
            link: Some(self.into()),
        }
    }
}

impl ::std::fmt::Display for ErrLink {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let mut err = Ok(());
        for e in LinkIter::iter_over(self) {
            err = writeln!(f, "{}", e);
        }
        err
    }
}

impl<E> From<E> for ErrLink where E: ::std::error::Error + 'static {
    fn from(e: E) -> Self {
        ErrLink {
            error: e.into(),
            link: None,
        }
    }
}

struct LinkIter<'a> {
    error: Option<&'a ::std::error::Error>,
    link: Option<&'a ErrLink>,
}

impl<'a> LinkIter<'a> {
    fn iter_over(elink: &'a ErrLink) -> Self {
        LinkIter {
            error: Some(elink.error.as_ref()),
            link: elink.link.as_ref().map(|v| v.as_ref()),
        }
    }
}

impl<'a> Iterator for LinkIter<'a> {
    type Item = &'a ::std::error::Error;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.error;
        match self.link {
            Some(val)   => *self = LinkIter::iter_over(val),
            None        => self.error = None,
        };
        ret
    }
}

/// Type signature that all function returning an error to be used
/// in a chain can use
pub type Kresult<T> = ::std::result::Result<T, ErrLink>;

/// The trait that adds the chain_err function to any result carrying
/// anything that impls the Error trait
pub trait ErrorChain<T> {
    fn chain_err<F, E>(self, f: F) -> Kresult<T>
        where F: FnOnce() -> E, E: ::std::error::Error + 'static;
}

impl<T, E> ErrorChain<T> for ::std::result::Result<T, E> where E: Into<ErrLink> {
    fn chain_err<F, E2>(self, f: F) -> Kresult<T>
        where F: FnOnce() -> E2, E2: ::std::error::Error + 'static {
            self.map_err(|e| {
                e.into().attach_links(f())
            })
        }
}

macro_rules! make_error {
    ( $name:ident $(< $($a:tt),* ; $($T:tt $(: $L:tt)*),* >)* ; $msg:expr ; $( $param:ident : $val:ty),* ) => {
        #[derive(Debug)]
        pub struct $name$(< $($a,)* $($T : $($L +)* ::std::fmt::Debug + ::std::fmt::Display,)* >)*{$( $param : $val,)*}
        impl$(< $($a,)* $($T : ::std::fmt::Debug + ::std::fmt::Display,)* >)* $name$(< $($a,)* $($T,)* >)* {
            pub fn new($( $param: $val, )*) -> Self {
                $name { $( $param: $param, )* }
            }
        }
        impl$(< $($a,)* $($T : ::std::fmt::Debug + ::std::fmt::Display,)* >)* ::std::fmt::Display for $name$(< $($a,)* $($T,)* >)* {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(fmt, $msg, $( self.$param, )*)
            }
        }
        impl$(< $($a,)* $($T : ::std::fmt::Debug + ::std::fmt::Display,)* >)* ::std::error::Error for $name$(< $($a,)* $($T,)* >)* {
            fn description(&self) -> &str {
                concat!(concat!("Error: ", stringify!($name)))
            }
        }
    }
}
