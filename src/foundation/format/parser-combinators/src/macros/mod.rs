mod dispatch;
mod seq;
mod unordered_seq;

#[cfg(test)]
include!("../../../../../../test/foundation/format/parser-combinators/unit/src/macros/mod/43.rs");

macro_rules! impl_partial_eq {
    ($lhs:ty, $rhs:ty) => {
        #[allow(unused_lifetimes)]
        impl<'a> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                let l = self;
                let r: &Self = other.as_ref();
                PartialEq::eq(l, r)
            }
        }

        #[allow(unused_lifetimes)]
        impl<'a> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(other, self)
            }
        }
    };
}

macro_rules! impl_partial_ord {
    ($lhs:ty, $rhs:ty) => {
        #[allow(unused_lifetimes)]
        impl<'a> PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<Ordering> {
                let l = self;
                let r: &Self = other.as_ref();
                PartialOrd::partial_cmp(l, r)
            }
        }

        #[allow(unused_lifetimes)]
        impl<'a> PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<Ordering> {
                PartialOrd::partial_cmp(other, self)
            }
        }
    };
}

#[path = "../../../../../../test/foundation/format/parser-combinators/unit/src/macros/tests.rs"]
#[cfg(test)]
mod tests;
