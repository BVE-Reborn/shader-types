//! Vector and Matrix types that are properly aligned for use in std140 uniforms.
//!
//! All the types in this library have the same alignment and size as the equivilant glsl type in the
//! default mode (std140).
//!
//! This fixes the padding within members of structs but padding between members needs to be minded.
//! The types in [`padding`] are there to make this easier.
//!
//! Vectors are constructable to/from an array of their underlying type. Matrices are constructable
//! to/from both 1d and 2d arrays as well as an array of the underlying _vector_ type. (eg. [`Mat2`] can be
//! constructed from `[Vec2; 2]`)
//!
//! # Example
//!
//! For the following glsl:
//!
//! ```glsl
//! layout(set = 0, binding = 0) uniform Block {
//!     mat4 mvp;
//!     vec3 position;
//!     vec3 normal;
//!     vec2 uv;
//! }
//! ```
//!
//! This struct is rife with padding. However it's now easy to mind the padding:
//!
//! ```rust
//! use shader_types::{Vec2, Vec3, Mat4};
//! use shader_types::padding::Pad2Float;
//!
//! // Definition
//! #[repr(C)]
//! #[derive(Copy, Clone)]
//! struct UniformBlock {
//!     mvp: Mat4, // 16 align + 64 size
//!     position: Vec3, // 16 align + 12 size
//!     normal: Vec3, // 16 align + 12 size
//!     uv: Vec2, // 8 align + 8 size
//!     _padding: Pad2Float, // Struct is 16 byte aligned, so we need (the space of) 2 more floats.
//! }
//!
//! fn generate_mvp() -> [f32; 16] {
//!     // ...
//! #     unsafe { std::mem::zeroed() }
//! }
//!
//! // Construction
//! let block = UniformBlock {
//!     // Anything that can be converted to a [f32; 16] or [[f32; 4]; 4] works
//!     mvp: Mat4::from(generate_mvp()),
//!     position: Vec3::new([0.0, 1.0, 2.0]), // `from` also works
//!     normal: Vec3::new([-2.0, 2.0, 3.0]),
//!     uv: Vec2::new([0.0, 1.0]),
//!     _padding: Pad2Float::new(), // `default` also works
//! };
//!
//! // Supports bytemuck with the `bytemuck` feature
//! unsafe impl bytemuck::Zeroable for UniformBlock {}
//! unsafe impl bytemuck::Pod for UniformBlock {}
//!
//! let block_u8: &[u8] = bytemuck::cast_slice(&[block]);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

macro_rules! define_vector {
    ($name:ident, $mint_type:ident, $align:literal, $ty:ty, $count:literal<- $doc:literal) => {
        #[doc = $doc]
        #[repr(C, align($align))]
        #[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
        pub struct $name {
            pub inner: [$ty; $count],
        }

        #[cfg(feature = "bytemuck")]
        unsafe impl bytemuck::Zeroable for $name {}
        #[cfg(feature = "bytemuck")]
        unsafe impl bytemuck::Pod for $name {}

        impl $name {
            #[inline(always)]
            pub fn new(inner: [$ty; $count]) -> Self {
                Self {
                    inner,
                }
            }
        }

        #[cfg(feature = "mint")]
        impl From<mint::$mint_type<$ty>> for $name {
            #[inline(always)]
            fn from(other: mint::$mint_type<$ty>) -> Self {
                // Mint's types do not implement From for arrays, only Into.
                let inner: [$ty; $count] = other.into();

                Self { inner }
            }
        }

        impl From<[$ty; $count]> for $name {
            #[inline(always)]
            fn from(inner: [$ty; $count]) -> Self {
                Self {
                    inner,
                }
            }
        }

        #[cfg(feature = "mint")]
        impl From<$name> for mint::$mint_type<$ty> {
            #[inline(always)]
            fn from(other: $name) -> Self {
                other.inner.into()
            }
        }

        impl From<$name> for [$ty; $count] {
            #[inline(always)]
            fn from(other: $name) -> Self {
                other.inner
            }
        }
    };
}

define_vector!(Vec2, Vector2, 8, f32, 2 <- "Vector of 2 f32s. Alignment 8, size 16.");
define_vector!(Vec3, Vector3, 16, f32, 3 <- "Vector of 3 f32s. Alignment 16, size 24.");
define_vector!(Vec4, Vector4, 16, f32, 4 <- "Vector of 4 f32s. Alignment 16, size 32.");
define_vector!(DVec2, Vector2, 16, f64, 2 <- "Vector of 2 f64s. Alignment 16, size 32.");
define_vector!(DVec3, Vector3, 32, f64, 3 <- "Vector of 3 f64s. Alignment 32, size 48.");
define_vector!(DVec4, Vector4, 32, f64, 4 <- "Vector of 4 f64s. Alignment 32, size 64.");
define_vector!(UVec2, Vector2, 8, u32, 2 <- "Vector of 2 u32s. Alignment 8, size 16.");
define_vector!(UVec3, Vector3, 16, u32, 3 <- "Vector of 3 u32s. Alignment 16, size 24.");
define_vector!(UVec4, Vector4, 16, u32, 4 <- "Vector of 4 u32s. Alignment 16, size 32.");
define_vector!(IVec2, Vector2, 8, i32, 2 <- "Vector of 2 i32s. Alignment 8, size 16.");
define_vector!(IVec3, Vector3, 16, i32, 3 <- "Vector of 3 i32s. Alignment 16, size 24.");
define_vector!(IVec4, Vector4, 16, i32, 4 <- "Vector of 4 i32s. Alignment 16, size 32.");

macro_rules! define_matrix {
    ($name:ident, $mint_type:ident, $align:literal, $inner_ty:ty, $ty:ty, $count_x:literal, $count_y:literal, $padding:literal -> $($idx:literal),* <- $doc:literal) => {
        #[doc = $doc]
        #[repr(C, align($align))]
        #[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
        pub struct $name {
            pub inner: [$ty; $count_y],
            _padding: [u8; $padding],
        }

        #[cfg(feature = "bytemuck")]
        unsafe impl bytemuck::Zeroable for $name {}
        #[cfg(feature = "bytemuck")]
        unsafe impl bytemuck::Pod for $name {}

        impl $name {
            #[inline(always)]
            pub fn new(inner: [$ty; $count_y]) -> Self {
                Self { inner, _padding: [0; $padding] }
            }
        }

        #[cfg(feature = "mint")]
        impl From<mint::$mint_type<$inner_ty>> for $name {
            #[inline(always)]
            fn from(other: mint::$mint_type<$inner_ty>) -> Self {
                // Mint's types do not implement From for arrays, only Into.
                let as_arr: [$inner_ty; $count_x * $count_y] = other.into();
                as_arr.into()
            }
        }

        impl From<[$ty; $count_y]> for $name {
            #[inline(always)]
            fn from(inner: [$ty; $count_y]) -> Self {
                Self { inner, _padding: [0; $padding]  }
            }
        }

        impl From<[$inner_ty; $count_x * $count_y]> for $name {
            #[inline(always)]
            fn from(inner: [$inner_ty; $count_x * $count_y]) -> Self {
                let d2: [[$inner_ty; $count_x]; $count_y] = unsafe { core::mem::transmute(inner) };
                Self {
                    inner: [$(<$ty>::from(d2[$idx])),*],
                    _padding: [0; $padding],
                }
            }
        }

        impl From<[[$inner_ty; $count_x]; $count_y]> for $name {
            #[inline(always)]
            fn from(inner: [[$inner_ty; $count_x]; $count_y]) -> Self {
                Self {
                    inner: [$(<$ty>::from(inner[$idx])),*],
                    _padding: [0; $padding],
                }
            }
        }

        #[cfg(feature = "mint")]
        impl From<$name> for mint::$mint_type<$inner_ty> {
            #[inline(always)]
            fn from(other: $name) -> Self {
                let as_arr = <[[$inner_ty; $count_x]; $count_y]>::from(other);
                as_arr.into()
            }
        }

        impl From<$name> for [$ty; $count_y] {
            #[inline(always)]
            fn from(other: $name) -> Self {
                other.inner
            }
        }

        impl From<$name> for [$inner_ty; $count_x * $count_y] {
            #[inline(always)]
            fn from(other: $name) -> Self {
                let d2: [[$inner_ty; $count_x]; $count_y] = [$(<[$inner_ty; $count_x]>::from(other.inner[$idx])),*];
                unsafe { core::mem::transmute(d2) }
            }
        }

        impl From<$name> for [[$inner_ty; $count_x]; $count_y] {
            #[inline(always)]
            fn from(other: $name) -> Self {
                [$(<[$inner_ty; $count_x]>::from(other.inner[$idx])),*]
            }
        }
    };
}

define_matrix!(Mat2x2, ColumnMatrix2, 8, f32, Vec2, 2, 2, 0 -> 0, 1 <- "Matrix of f32s with 2 columns and 2 rows. Alignment 8, size 16.");
define_matrix!(Mat2x3, ColumnMatrix2x3, 8, f32, Vec2, 2, 3, 8 -> 0, 1, 2 <- "Matrix of f32s with 2 columns and 3 rows. Alignment 8, size 32.");
define_matrix!(Mat2x4, ColumnMatrix2x4, 8, f32, Vec2, 2, 4, 0 -> 0, 1, 2, 3 <- "Matrix of f32s with 2 columns and 4 rows. Alignment 8, size 32.");

define_matrix!(Mat3x2, ColumnMatrix3x2, 16, f32, Vec3, 3, 2, 4 -> 0, 1 <- "Matrix of f32s with 3 columns and 2 rows. Alignment 16, size 32.");
define_matrix!(Mat3x3, ColumnMatrix3, 16, f32, Vec3, 3, 3, 4 -> 0, 1, 2 <- "Matrix of f32s with 3 columns and 3 rows. Alignment 16, size 48.");
define_matrix!(Mat3x4, ColumnMatrix3x4, 16, f32, Vec3, 3, 4, 4 -> 0, 1, 2, 3 <- "Matrix of f32s with 3 columns and 4 rows. Alignment 16, size 64.");

define_matrix!(Mat4x2, ColumnMatrix4x2, 16, f32, Vec4, 4, 2, 0 -> 0, 1 <- "Matrix of f32s with 4 columns and 2 rows. Alignment 16, size 32.");
define_matrix!(Mat4x3, ColumnMatrix4x3, 16, f32, Vec4, 4, 3, 0 -> 0, 1, 2 <- "Matrix of f32s with 4 columns and 3 rows. Alignment 16, size 48.");
define_matrix!(Mat4x4, ColumnMatrix4, 16, f32, Vec4, 4, 4, 0 -> 0, 1, 2, 3 <- "Matrix of f32s with 4 columns and 4 rows. Alignment 16, size 64.");

/// Matrix of f32s with 2 columns and 2 rows. Alignment 8, size 16.
pub type Mat2 = Mat2x2;
/// Matrix of f32s with 3 columns and 3 rows. Alignment 16, size 48.
pub type Mat3 = Mat3x3;
/// Matrix of f32s with 4 columns and 4 rows. Alignment 16, size 64.
pub type Mat4 = Mat4x4;

define_matrix!(DMat2x2, ColumnMatrix2, 16, f64, DVec2, 2, 2, 0 -> 0, 1 <- "Matrix of f64s with 2 columns and 2 rows. Alignment 16, size 32.");
define_matrix!(DMat2x3, ColumnMatrix2x3, 16, f64, DVec2, 2, 3, 0 -> 0, 1, 2 <- "Matrix of f64s with 2 columns and 3 rows. Alignment 16, size 48.");
define_matrix!(DMat2x4, ColumnMatrix2x4, 16, f64, DVec2, 2, 4, 0 -> 0, 1, 2, 3 <- "Matrix of f64s with 2 columns and 4 rows. Alignment 16, size 64.");

define_matrix!(DMat3x2, ColumnMatrix3x2, 32, f64, DVec3, 3, 2, 0 -> 0, 1 <- "Matrix of f64s with 3 columns and 2 rows. Alignment 32, size 64.");
define_matrix!(DMat3x3, ColumnMatrix3, 32, f64, DVec3, 3, 3, 0 -> 0, 1, 2 <- "Matrix of f64s with 3 columns and 3 rows. Alignment 32, size 96.");
define_matrix!(DMat3x4, ColumnMatrix3x4, 32, f64, DVec3, 3, 4, 0 -> 0, 1, 2, 3 <- "Matrix of f64s with 3 columns and 4 rows. Alignment 32, size 128.");

define_matrix!(DMat4x2, ColumnMatrix4x2, 32, f64, DVec4, 4, 2, 0 -> 0, 1 <- "Matrix of f64s with 4 columns and 2 rows. Alignment 32, size 64.");
define_matrix!(DMat4x3, ColumnMatrix4x3, 32, f64, DVec4, 4, 3, 0 -> 0, 1, 2 <- "Matrix of f64s with 4 columns and 3 rows. Alignment 32, size 96.");
define_matrix!(DMat4x4, ColumnMatrix4, 32, f64, DVec4, 4, 4, 0 -> 0, 1, 2, 3 <- "Matrix of f64s with 4 columns and 4 rows. Alignment 32, size 128.");

/// Matrix of f64s with 2 columns and 3 rows. Alignment 16, size 48.
pub type DMat2 = DMat2x2;
/// Matrix of f64s with 3 columns and 3 rows. Alignment 32, size 96.
pub type DMat3 = DMat3x3;
/// Matrix of f64s with 4 columns and 4 rows. Alignment 32, size 128.
pub type DMat4 = DMat4x4;

/// Correctly sized padding helpers.
pub mod padding {
    macro_rules! define_padding {
        ($name:ident, $count:literal <- $doc:literal) => {
            #[doc = $doc]
            #[repr(C)]
            #[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
            pub struct $name {
                _padding: [u8; $count],
            }

            #[cfg(feature = "bytemuck")]
            unsafe impl bytemuck::Zeroable for $name {}
            #[cfg(feature = "bytemuck")]
            unsafe impl bytemuck::Pod for $name {}

            impl $name {
                #[inline(always)]
                pub fn new() -> Self {
                    Self::default()
                }
            }
        };
    }

    define_padding!(Pad1Float, 4 <- "Padding the size of a single float/uint/int. 4 bytes.");
    define_padding!(Pad2Float, 8 <- "Padding the size of two floats/uints/ints. 8 bytes.");
    define_padding!(Pad3Float, 12 <- "Padding the size of three floats/uints/ints. 12 bytes.");
    define_padding!(Pad4Float, 16 <- "Padding the size of four floats/uints/ints. 16 bytes.");
    define_padding!(Pad1Double, 8 <- "Padding the size of a single double. 8 bytes.");
    define_padding!(Pad2Double, 16 <- "Padding the size of two doubles. 16 bytes.");
    define_padding!(Pad3Double, 24 <- "Padding the size of three doubles. 24 bytes.");
    define_padding!(Pad4Double, 32 <- "Padding the size of four doubles. 32 bytes.");
}
