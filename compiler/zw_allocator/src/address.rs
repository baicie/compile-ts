use std::{ops::Add, ptr};

use crate::Box;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address(usize);

impl Address {
    pub const DUMMY: Self = Self(0);

    #[inline]
    pub fn from_ptr<T>(p: *const T) -> Self {
        Self(p as usize)
    }
}

pub trait GetAddress {
    fn address(&self)-> Address
}

impl <T> GetAddress for Box<'_,T> {
    #[inline]
    fn address(&self)-> Address {
        Address::from_ptr(ptr::addr_of!(**self))
    }
}

impl GetAddress for Address {
    #[inline]
    fn address(&self)-> Address {
        **self
    }
}