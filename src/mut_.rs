use std::kinds::marker;
use std::mem;
use base;
use base::Strided as Base;

/// A mutable strided slice. This is equivalent to `&mut [T]`, that
/// only refers to every `n`th `T`.
///
/// This can be viewed as an immutable strided slice via the `Deref`
/// implementation, and so many methods are available through that
/// type.
///
/// Many functions in this API take `self` and consume it. The
/// `reborrow` method is a key part of ensuring that ownership doesn't
/// disappear completely: it converts a reference `&'b mut
/// MutStrided<'a, T>` into a `MutStrided<'b, T>`, that is, gives a
/// by-value slice with a shorter lifetime. This can then be passed
/// directly into the functions that consume `self` without losing
/// control of the original slice.
#[repr(C)]
pub struct Strided<'a,T: 'a> {
    base: Base<'a, T>,
    _marker: marker::NoCopy,
}

impl<'a, T> Strided<'a, T> {
    #[inline(always)]
    fn new_raw(base: Base<'a, T>) -> Strided<'a, T> {
        Strided {
            base: base,
            _marker: marker::NoCopy
        }
    }

    /// Creates a new strided slice directly from a conventional
    /// slice. The return value has stride 1.
    #[inline(always)]
    pub fn new(x: &'a mut [T]) -> Strided<'a, T> {
        Strided::new_raw(Base::new(x.as_mut_ptr(), x.len(), 1))
    }

    /// Returns the number of elements accessible in `self`.
    #[inline(always)]
    pub fn len(&self) -> uint {
        self.base.len()
    }
    /// Returns the offset between successive elements of `self` as a
    /// count of *elements*, not bytes.
    #[inline(always)]
    pub fn stride(&self) -> uint {
        self.base.stride() / mem::size_of::<T>()
    }

    /// Returns a pointer to the first element of this strided slice.
    ///
    /// NB. one must be careful since only every `self.stride()`th
    /// element is guaranteed to have unique access via this object;
    /// the others may be under the control of some other strided
    /// slice.
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.base.as_mut_ptr()
    }

    /// Creates a temporary copy of this strided slice.
    ///
    /// This is an explicit form of the reborrowing the compiler does
    /// implicitly for conventional `&mut` pointers. This is designed
    /// to allow the by-value `self` methods to be used without losing
    /// access to the slice.
    #[inline(always)]
    pub fn reborrow<'b>(&'b mut self) -> Strided<'b, T> {
        Strided::new_raw(self.base)
    }

    /// Breaks this strided slice into two strided slices pointing to
    /// alternate elements.
    ///
    /// That is, it doubles the stride and (approximately) halves the
    /// length. A slice pointing to values `[1, 2, 3, 4, 5]` becomes
    /// two slices `[1, 3, 5]` and `[2, 4]`. This is guaranteed to
    /// succeed even for mismatched lengths, and even if `self` has
    /// only zero or one elements.
    #[inline]
    pub fn substrides2(self) -> (Strided<'a, T>, Strided<'a, T>) {
        let (l, r) = self.base.substrides2();
        (Strided::new_raw(l), Strided::new_raw(r))
    }

    /// Returns an iterator over `n` strided subslices of `self` each
    /// pointing to every `n`th element, starting at successive
    /// offsets.
    ///
    /// Calling `substrides(3)` on a slice pointing to `[1, 2, 3, 4, 5, 6,
    /// 7]` will yield, in turn, `[1, 4, 7]`, `[2, 5]` and finally
    /// `[3, 6]`. Like with `split2` this is guaranteed to succeed
    /// (return `n` strided slices) even if `self` has fewer than `n`
    /// elements.
    #[inline]
    pub fn substrides(self, n: uint) -> Substrides<'a, T> {
        Substrides {
            base: self.base.substrides(n),
            _marker: marker::NoCopy,
        }
    }
    /// Returns a reference to the `n`th element of `self`, or `None`
    /// if `n` is out-of-bounds.
    #[inline]
    pub fn get_mut<'b>(&'b mut self, n: uint) -> Option<&'b mut T> {
        self.base.get_mut(n).map(|r| &mut *r)
    }

    /// Returns an iterator over references to each successive element
    /// of `self`.
    ///
    /// See also `into_iter` which gives the references the maximum
    /// possible lifetime at the expense of consume the slice.
    #[inline]
    pub fn iter_mut<'b>(&'b mut self) -> ::MutItems<'b, T> {
        self.reborrow().into_iter()
    }

    /// Returns an iterator over reference to each successive element
    /// of `self`, with the maximum possible lifetime.
    ///
    /// See also `iter_mut` which avoids consuming `self` at the
    /// expense of shorter lifetimes.
    #[inline]
    pub fn into_iter(mut self) -> ::MutItems<'a, T> {
        self.base.iter_mut()
    }

    /// Returns a strided slice containing only the elements from
    /// indices `from` (inclusive) to `to` (exclusive).
    ///
    /// # Panic
    ///
    /// Panics if `from > to` or if `to > self.len()`.
    #[inline]
    pub fn slice(self, from: uint, to: uint) -> Strided<'a, T> {
        Strided::new_raw(self.base.slice(from, to))
    }
    /// Returns a strided slice containing only the elements from
    /// index `from` (inclusive).
    ///
    /// # Panic
    ///
    /// Panics if `from > self.len()`.
    #[inline]
    pub fn slice_from(self, from: uint) -> Strided<'a, T> {
        Strided::new_raw(self.base.slice_from(from))
    }
    /// Returns a strided slice containing only the elements to
    /// index `to` (exclusive).
    ///
    /// # Panic
    ///
    /// Panics if `to > self.len()`.
    #[inline]
    pub fn slice_to(self, to: uint) -> Strided<'a, T> {
        Strided::new_raw(self.base.slice_to(to))
    }
    /// Returns two strided slices, the first with elements up to
    /// `idx` (exclusive) and the second with elements from `idx`.
    ///
    /// This is semantically equivalent to `(self.slice_to(idx),
    /// self.slice_from(idx))`.
    ///
    /// # Panic
    ///
    /// Panics if `idx > self.len()`.
    #[inline]
    pub fn split_at(self, idx: uint) -> (Strided<'a, T>, Strided<'a, T>) {
        let (l, r) = self.base.split_at(idx);
        (Strided::new_raw(l), Strided::new_raw(r))
    }
}

impl<'a, T> IndexMut<uint, T> for Strided<'a, T> {
    fn index_mut<'b>(&'b mut self, n: &uint) -> &'b mut T {
        self.get_mut(*n).expect("Strided.index_mut: index out of bounds")
    }
}

impl<'a, T> Deref<::imm::Strided<'a, T>> for Strided<'a, T> {
    fn deref<'b>(&'b self) -> &'b ::imm::Strided<'a, T> {
        unsafe { mem::transmute(self) }
    }
}

/// An iterator over `n` mutable substrides of a given stride, each of
/// which points to every `n`th element starting at successive
/// offsets.
pub struct Substrides<'a, T: 'a> {
    base: base::Substrides<'a, T>,
    _marker: marker::NoCopy
}

impl<'a, T> Iterator<Strided<'a, T>> for Substrides<'a, T> {
    fn next(&mut self) -> Option<Strided<'a, T>> {
        match self.base.next() {
            Some(s) => Some(Strided::new_raw(s)),
            None => None
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.base.size_hint()
    }
}

#[cfg(test)]
#[path="common_tests.rs"]
mod common_tests;

#[cfg(test)]
mod tests {
    use super::Strided;

    #[test]
    fn reborrow() {
        let v = &mut [1u8, 2, 3, 4, 5];
        let mut s = Strided::new(v);
        eq!(s.reborrow(), [1,2,3,4,5])
    }

    #[test]
    fn iter_mut() {
        let v = &mut [1u8, 2, 3, 4, 5];
        let s = Strided::new(v);
        eq!(s, [1, 2, 3, 4, 5], iter_mut);
    }

    #[test]
    fn get_mut() {
        macro_rules! test {
            ($input: expr, $expected: expr) => {{
                let mut e = $expected;
                for i in range(0, e.len() + 10) {
                    let expected = e.get_mut(i);
                    assert_eq!($input.get_mut(i).map(|x| *x), expected.as_ref().map(|x| **x));
                    match expected {
                        Some(x) => assert_eq!(*(&mut $input[i]), *x),
                        None => {}
                    }
                }
            }}
        }

        let v: &mut [u8] = [1, 2, 3, 4, 5, 6];
        let mut base = Strided::new(v);
        test!(base, [1,2,3,4,5,6]);
        let (mut l, mut r) = base.substrides2();
        test!(l, [1,3,5]);
        test!(r, [2,4,6])
    }
}