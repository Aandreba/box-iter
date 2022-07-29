use core::{alloc::Allocator, ptr::NonNull};
use alloc::boxed::Box;

pub trait BoxIntoIter {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    
    fn into_iter (self) -> Self::IntoIter;
}

impl<T, A: Allocator> BoxIntoIter for Box<[T], A> {
    type Item = T;
    type IntoIter = super::IntoIter<T, A>;

    #[inline(always)]
    fn into_iter (self) -> Self::IntoIter {
        let (ptr, alloc) = Box::into_raw_with_allocator(self);
        let range = unsafe { &mut *ptr }.as_mut_ptr_range();

        unsafe {
            super::IntoIter {
                ptr: NonNull::new_unchecked(ptr),
                range,
                alloc
            } 
        }
    }
}