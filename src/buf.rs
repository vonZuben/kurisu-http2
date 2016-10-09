//! Trait for type to implement if their underlying data
//! can be treated like a generic buffer of type T.
//! Intended to be used for other traits to help with default
//! method implementations
//!
//! This is really just a combination of AsRef and AsRefMut

/// The Buf Trait that that says a type contains a
/// buffer in its underlying memory and can be safely
/// exposed for exterior usage
pub trait Buf<T> {
    fn buf(&self) -> &[T];
    fn mut_buf(&mut self) -> &mut [T];
}

/// macro to automatically implement Buf for all listed types
/// with buffer type and member name pointer to the buffer given
macro_rules! impl_buf {
    ( $($buf_type:ty : $mem_name:ident => $($type_name:ty),+;)+ ) =>
        {
            $(
                $(
                    impl Buf<$buf_type> for $type_name {
                        fn buf(&self) -> &[$buf_type] {
                            &self.$mem_name
                        }
                        fn mut_buf(&mut self) -> &mut [$buf_type] {
                            &mut self.$mem_name
                        }
                    }
                 )*
             )*
        }
}

#[cfg(test)]
mod buf_tests {
    use super::Buf;

    struct TstImplBuf {
        buf: Vec<u8>,
    }

    impl_buf!( u8 : buf => TstImplBuf ; );

    #[test]
    fn test_buf() {
        let tb = TstImplBuf { buf: vec![1,2,3,4] };

        assert_eq!(&[1,2,3,4], tb.buf());
    }

    #[test]
    fn test_mut_buf() {
        let mut tmb = TstImplBuf { buf: vec![1,2,3,4] };

        {
            let mr = tmb.mut_buf();

            mr[0] = 0;
            mr[3] = 9;
        }

        assert_eq!(&[0,2,3,9], tmb.buf());
    }
}
