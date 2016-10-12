//! Trait for type that holds a unique mutable reference
//! to some underlying buffer. Used as the base for more
//! more complex types that point to and map out the buffer

/// The Buf Trait that that says a type contains an borrowed
/// buffer in its underlying memory and can be safely
/// exposed for exterior usage
pub trait Buf<'obj, 'buf, T> {
    fn buf(&'obj self) -> &'obj [T];
    fn mut_buf(&'obj mut self) -> &'obj mut [T];
    fn point_to(&'buf mut [T]) -> Self;
}

/// macro to automatically implement Buf for all listed types
/// with buffer type and member name pointer to the buffer given
macro_rules! impl_buf {
    ( $($buf_type:ty : $mem_name:ident => $($type_name:ident),+;)+ ) =>
        {
            $(
                $(
                    impl<'obj, 'buf> Buf<'obj, 'buf, $buf_type> for $type_name<'buf>
                        where 'buf: 'obj, [$buf_type]: 'buf{
                        fn buf(&'obj self) -> &'obj [$buf_type] {
                            &self.$mem_name
                        }
                        fn mut_buf(&'obj mut self) -> &'obj mut [$buf_type] {
                            &mut self.$mem_name
                        }
                        fn point_to(buf: &'buf mut [$buf_type]) -> Self {
                            $type_name { $mem_name: buf }
                        }
                    }
                 )*
             )*
        }
}

#[cfg(test)]
mod buf_tests {
    use super::Buf;

    struct TstImplBuf<'a> {
        buf: &'a mut [u8],
    }

    impl_buf!( u8 : buf => TstImplBuf ; );

    #[test]
    fn test_buf() {
        let mut buf = vec![1,2,3,4];
        let tb = TstImplBuf::point_to(&mut buf);

        assert_eq!(&[1,2,3,4], tb.buf());
    }

    #[test]
    fn test_mut_buf() {
        let mut buf = vec![1,2,3,4];
        let mut tmb = TstImplBuf::point_to(&mut buf);

        {
            let mr = tmb.mut_buf();

            mr[0] = 0;
            mr[3] = 9;
        }

        assert_eq!(&[0,2,3,9], tmb.buf());
    }
}
