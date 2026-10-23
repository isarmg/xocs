use core::ops::Range;

pub const fn clone<const N: usize>(bytes: &[u8]) -> [u8; N] {
    constfn_assert!(bytes.len() == N);
    let mut buf = [0; N];
    let mut i = 0;
    while i < bytes.len() {
        buf[i] = bytes[i];
        i += 1;
    }
    buf
}

pub const fn equal(lhs: &[u8], rhs: &[u8]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }
    let mut i = 0;
    while i < lhs.len() {
        if lhs[i] != rhs[i] {
            return false;
        }
        i += 1;
    }
    true
}

pub const fn subslice<T>(mut s: &[T], range: Range<usize>) -> &[T] {
    constfn_assert!(range.end >= range.start);

    let mut i = 0;
    let mut j = s.len();

    while i < range.start {
        match s {
            [_, xs @ ..] => {
                i += 1;
                s = xs;
            }
            _ => break,
        }
    }

    while j > range.end {
        match s {
            [xs @ .., _] => {
                j -= 1;
                s = xs;
            }
            _ => break,
        }
    }

    constfn_assert!(i == range.start);
    constfn_assert!(j == range.end);
    constfn_assert!(s.len() == j - i);
    s
}

#[cfg(test)]
include!("../../../../../test/foundation/text/compile-time-text/unit/src/bytes/1157.rs");

pub const fn merge<const N: usize>(mut buf: [u8; N], bytes: &[u8]) -> [u8; N] {
    constfn_assert!(N <= bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        buf[i] = bytes[i];
        i += 1;
    }
    buf
}

pub const fn reversed<const N: usize>(mut arr: [u8; N]) -> [u8; N] {
    let mut i = 0;
    while i * 2 < N {
        let a = arr[i];
        let b = arr[N - 1 - i];
        arr[i] = b;
        arr[N - 1 - i] = a;
        i += 1;
    }
    arr
}

#[cfg(test)]
include!("../../../../../test/foundation/text/compile-time-text/unit/src/bytes/1910.rs");

pub const fn advance(mut s: &[u8], count: usize) -> &[u8] {
    constfn_assert!(count <= s.len());
    let mut i = 0;
    while i < count {
        match s {
            [_, xs @ ..] => s = xs,
            _ => break,
        }
        i += 1;
    }
    constfn_assert!(i == count);
    s
}
