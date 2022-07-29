#![no_std]
#![feature(layout_for_ptr, allocator_api)]

pub(crate) extern crate alloc;

mod r#trait;
use alloc::alloc::Global;
pub use r#trait::*;
use core::{alloc::{Allocator, Layout}, ptr::NonNull, ops::Range, mem::needs_drop, iter::{FusedIterator}};

pub struct IntoIter<T, A: Allocator = Global> {
    pub(crate) ptr: NonNull<[T]>,
    pub(crate) range: Range<*mut T>,
    pub(crate) alloc: A
}

impl<T, A: Allocator> Iterator for IntoIter<T, A> {
    type Item = T;

    #[inline(always)]
    fn next (&mut self) -> Option<Self::Item> {
        self.nth(0)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline(always)]
    fn count(self) -> usize where Self: Sized {
        self.len()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> where Self: Sized {
        if self.range.end <= self.range.start {
            return None
        }

        let v = unsafe { core::ptr::read(self.range.end.sub(1)) };
        self.range.end = self.range.start;
        Some(v)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        unsafe {
            let ptr = self.range.start.add(n);

            if ptr >= self.range.end {
                self.range.start = self.range.end;
                return None
            }
    
            self.range.start = ptr.add(1);
            debug_assert!(!ptr.is_null());
            Some(core::ptr::read(ptr))
        }
    }
}

impl<T, A: Allocator> DoubleEndedIterator for IntoIter<T, A> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.nth_back(0)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        unsafe {
            self.range.end = self.range.end.sub(n + 1);
            let ptr = self.range.end;

            if ptr < self.range.start {
                self.range.end = self.range.start;
                return None
            }
    
            debug_assert!(!ptr.is_null());
            Some(core::ptr::read(ptr))
        }
    }
}

impl<T, A: Allocator> ExactSizeIterator for IntoIter<T, A> {
    #[inline(always)]
    fn len(&self) -> usize {
        ((self.range.end as usize) - (self.range.start as usize)) / core::mem::size_of::<T>()
    }
}

impl<T, A: Allocator> FusedIterator for IntoIter<T, A> {}

impl<T, A: Allocator> Drop for IntoIter<T, A> {
    #[inline]
    fn drop(&mut self) {
        if needs_drop::<T>() {
            let mut ptr = self.range.start;

            while ptr < self.range.end {
                unsafe {
                    core::ptr::drop_in_place(ptr);
                    ptr = ptr.add(1);
                }
            }
        }

        unsafe {
            let layout = Layout::for_value_raw(self.ptr.as_ptr());
            self.alloc.deallocate(self.ptr.cast(), layout);
        }
    }
}

unsafe impl<T: Send, A: Allocator + Send> Send for IntoIter<T, A> {}
unsafe impl<T: Sync, A: Allocator + Sync> Sync for IntoIter<T, A> {}

#[cfg(test)]
mod test {
    extern crate std;
    use std::{prelude::rust_2021::*, println};

    use alloc::vec;
    use crate::BoxIntoIter;

    #[test]
    fn test () {
        let array = vec!["hello".to_string(), "world".to_string()].into_boxed_slice().into_iter();
        println!("{:?}", array.last())
    }
}