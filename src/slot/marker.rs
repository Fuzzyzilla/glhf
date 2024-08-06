//! Markers for static information about the current binding.

use crate::sealed::Sealed;

/// Marker trait for the three states of "default" bindings -
/// [`IsDefault`], [`NotDefault`], and [`Unknown`].
///
/// The operations that are possible on a "default" object (name = 0) and
/// user-defined objects (name != 0) differ greatly. In some cases, such as
/// buffers, the 0 name is functionally "null" and thus few operations are possible.
/// In others, such as Textures, the 0 name refers to a GL-owned default object.
///
/// When a slot is of [`Unknown`] defaultness, operations are limited to the intersection
/// of the two capabilities. This only occurs when the binding is "inherited" previous
/// GL calls and thus cannot be statically known.
pub trait Defaultness: Sealed + 'static {}

/// Not statically known whether the zero object or a non-zero object is bound.
///
/// See [`Defaultness`] for more information.
#[derive(Debug)]
pub struct Unknown;
impl crate::sealed::Sealed for Unknown {}
impl Defaultness for Unknown {}
/// Statically known that the default object, 0, is bound.
///
/// See [`Defaultness`] for more information.
#[derive(Debug)]
pub struct IsDefault;
impl crate::sealed::Sealed for IsDefault {}
impl Defaultness for IsDefault {}
/// Statically known that a user-defined object, not 0, is bound.
///
/// See [`Defaultness`] for more information.
#[derive(Debug)]
pub struct NotDefault;
impl crate::sealed::Sealed for NotDefault {}
impl Defaultness for NotDefault {}
