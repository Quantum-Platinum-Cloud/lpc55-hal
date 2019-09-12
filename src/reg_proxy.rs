//! Makes registers ownable and movable
//!
//! The register code generated by svd2rust doesn't allows us to move and own
//! registers. We can only have shared references to them. This becomes
//! inconvenient, if we want to split a peripheral, so multiple components of an
//! API can access it, as every component requires a lifetime then.
//!
//! This module works around this limitation, by introducing a proxy struct that
//! provides access to a register.

// Context: https://github.com/rust-embedded/svd2rust/issues/213

use core::marker::PhantomData;
use core::ops::Deref;

/// Implemented for registers that `RegProxy` can proxy
///
/// Use the `reg!` macro to implement this trait for a register from a crate
/// generated by svd2rust.
///
/// Safety: The pointer returned by `get` must be valid for the duration of the program.
pub unsafe trait Reg {
    /// The type that `RegProxy` should derefence to
    ///
    /// If only one instance of the register exists, this should be `Self`.
    /// If the same type in the svd2rust API is used to represent registers at
    /// multiple memory locations, this trait must be implemented for a type
    /// that represents a specific register at a specific location, and `Target`
    /// must be the common type.
    type Target;

    /// Return a pointer to the memory location of the register
    fn get() -> *const Self::Target;
}

#[macro_export]
macro_rules! reg {
    ($ty:ident, $target:ty, $peripheral:path, $field:ident) => {
        unsafe impl $crate::reg_proxy::Reg for $ty {
            type Target = $target;

            fn get() -> *const Self::Target {
                unsafe { &(*<$peripheral>::ptr()).$field as *const $ty }
            }
        }
    };
}

// Example:
//
// unsafe impl crate::reg_proxy::Reg for AHBCLKCTRL0 {
//     type Target = AHBCLKCTRL0;
//     fn get() -> *const Self::Target {
//         unsafe { &(*<raw::SYSCON>::ptr()).ahbclkctrl0 as *const _ }
//     }
// }
//
// reg!(AHBCLKCTRL0, AHBCLKCTRL0, raw::SYSCON, ahbclkctrl0);
// reg!(raw::AHBCLKCTRL0, raw::AHBCLKCTRL0, raw::SYSCON, ahbclkctrl0);

// reg!([DIRSET], DIREST, raw::GPIO, dirset);

/// A proxy object for a register
///
/// This proxy can be moved and owned. Access via `Deref`.
pub struct RegProxy<T>
where
    T: Reg,
{
    _marker: PhantomData<*const T>,
}

impl<T> RegProxy<T>
where
    T: Reg,
{
    /// Create a new proxy object
    #[allow(dead_code)]
    pub fn new() -> Self {
        RegProxy {
            _marker: PhantomData,
        }
    }
}

unsafe impl<T> Send for RegProxy<T> where T: Reg {}

impl<T> Deref for RegProxy<T>
where
    T: Reg,
{
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        // As long as `T` upholds the safety restrictions laid out in the
        // documentation of `Reg`, this should be safe. The pointer is valid for
        // the duration of the program. That means:
        // 1. It can always be dereferenced, so casting to a reference is safe.
        // 2. It is essentially `'static`, so casting to any lifetime is safe.
        unsafe { &*T::get() }
    }
}

pub unsafe trait RegCluster {
    /// The type that `RegProxy` should derefence to
    ///
    /// If only one instance of the register exists, this should be `Self`.
    /// If the same type in the svd2rust API is used to represent registers at
    /// multiple memory locations, this trait must be implemented for a type
    /// that represents a specific register at a specific location, and `Target`
    /// must be the common type.
    type Target;

    /// Return a pointer to the memory location of the register
    fn get() -> *const [Self::Target];
}

#[macro_export]
macro_rules! reg_cluster {
    ($ty:ident, $target:ty, $peripheral:path, $field:ident) => {
        unsafe impl $crate::reg_proxy::RegCluster for $ty {
            type Target = $target;

            fn get() -> *const [Self::Target] {
                unsafe { &(*<$peripheral>::ptr()).$field as *const [$ty] }
            }
        }
    };
}

// Example:
//
// unsafe impl crate::reg_proxy::RegCluster for DIRSET {
//     type Target = DIRSET;
//     fn get() -> *const [Self::Target] {
//         unsafe { &(*<raw::GPIO>::ptr()).dirset as *const [DIRSET] }
//     }
// }
//
// reg_cluster!(DIRSET, DIRSET, raw::GPIO, dirset);

// For clusters, e.g. GPIO's set, clr and dirset
pub struct RegClusterProxy<T>
where
    T: RegCluster,
{
    _marker: PhantomData<*const [T]>,
}

impl<T> RegClusterProxy<T>
where
    T: RegCluster,
{
    /// Create a new proxy object
    pub fn new() -> Self {
        RegClusterProxy {
            _marker: PhantomData,
        }
    }
}

unsafe impl<T> Send for RegClusterProxy<T> where T: RegCluster {}

impl<T> Deref for RegClusterProxy<T>
where
    T: RegCluster,
{
    type Target = [T::Target];

    fn deref(&self) -> &Self::Target {
        unsafe { &*T::get() }
    }
}
