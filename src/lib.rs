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
//!     int constants[3];
//! };
//! ```
//!
//! This struct is rife with padding. However it's now easy to mind the padding:
//!
//! ```rust
//! use shader_types::{Vec2, Vec3, Mat4, ArrayMember};
//!
//! // Definition
//! #[repr(C)]
//! #[derive(Copy, Clone)]
//! struct UniformBlock {
//!     mvp: Mat4, // 16 align + 64 size
//!     position: Vec3, // 16 align + 12 size
//!     normal: Vec3, // 16 align + 12 size
//!     uv: Vec2, // 8 align + 8 size
//!     constants: [ArrayMember<i32>; 3] // 3x 16 align + 4 size
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
//!     constants: [ArrayMember(0), ArrayMember(1), ArrayMember(2)]
//! };
//!
//! // Supports bytemuck with the `bytemuck` feature
//! unsafe impl bytemuck::Zeroable for UniformBlock {}
//! unsafe impl bytemuck::Pod for UniformBlock {}
//!
//! let block_u8: &[u8] = bytemuck::cast_slice(&[block]);
//! ```
//!
//! # MSRV
//!
//! Rust 1.34

#![cfg_attr(not(feature = "std"), no_std)]

macro_rules! define_vectors {
    ( $(( $name:ident, $mint_name:ident, $prim:ident * $count:literal, align: $align:literal, size: $size:literal ),)* ) => {
        $(
            define_vectors!(@impl
                $name,
                mint::$mint_name<$prim>,
                $align,
                $prim,
                $count,
                concat!(
                    "Vector of ", stringify!($count), " `", stringify!($prim), "` values. ",
                    "Has size ", stringify!($size), " and alignment ", stringify!($align), "."
                ),
                concat!(
                    "Construct a `", stringify!($name), "` from any type which is convertable into a ",
                    "`mint::", stringify!($mint_name), "<", stringify!($prim), ">`."
                )
            );
        )*
    };

    (@impl $name:ident, $mint_type:ty, $align:literal, $ty:ty, $count:literal, $doc:expr, $mint_doc:expr) => {
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

            #[cfg(feature = "mint")]
            #[doc = $mint_doc]
            #[inline(always)]
            pub fn from_mint<T: Into<$mint_type>>(value: T) -> Self {
                Self::from(value.into())
            }
        }

        #[cfg(feature = "mint")]
        impl From<$mint_type> for $name {
            #[inline(always)]
            fn from(other: $mint_type) -> Self {
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
        impl From<$name> for $mint_type {
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

define_vectors! {
    (Vec2, Vector2, f32 * 2, align: 8, size: 16),
    (Vec3, Vector3, f32 * 3, align: 16, size: 24),
    (Vec4, Vector4, f32 * 4, align: 16, size: 32),
    (DVec2, Vector2, f64 * 2, align: 16, size: 32),
    (DVec3, Vector3, f64 * 3, align: 32, size: 48),
    (DVec4, Vector4, f64 * 4, align: 32, size: 64),
    (UVec2, Vector2, u32 * 2, align: 8, size: 16),
    (UVec3, Vector3, u32 * 3, align: 16, size: 24),
    (UVec4, Vector4, u32 * 4, align: 16, size: 32),
    (IVec2, Vector2, i32 * 2, align: 8, size: 16),
    (IVec3, Vector3, i32 * 3, align: 16, size: 24),
    (IVec4, Vector4, i32 * 4, align: 16, size: 32),
}

macro_rules! define_matrices {
    ( $(( $name:ident, $mint_name:ident, $prim_ty:ty, $row_ty:ty, $rows:literal * $cols:literal, align: $align:literal, size: $size:literal, pad: $pad:literal, [$($idx:literal),*] ),)* ) => {
        $(
            define_matrices!(@impl
                $name,
                mint::$mint_name<$prim_ty>,
                $align,
                $prim_ty,
                $row_ty,
                $rows,
                $cols,
                $pad,
                [$( $idx ),*],
                concat!(
                    "Matrix of `", stringify!($prim_ty), "` values with ", stringify!($rows), " rows and ", stringify!($cols), " columns. ",
                    "Has size ", stringify!($size), " and alignment ", stringify!($align), "."
                ),
                concat!(
                    "Construct a `", stringify!($name), "` from any type which is convertable into a ",
                    "`mint::", stringify!($mint_name), "<", stringify!($prim_ty), ">`."
                )
            );
        )*
    };

    (@impl $name:ident, $mint_type:ty, $align:literal, $inner_ty:ty, $ty:ty, $count_x:literal, $count_y:literal, $padding:literal, [$( $idx:literal ),*], $doc:expr, $mint_doc:expr) => {
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

            #[cfg(feature = "mint")]
            #[doc = $mint_doc]
            #[inline(always)]
            pub fn from_mint<T: Into<$mint_type>>(value: T) -> Self {
                Self::from(value.into())
            }
        }

        #[cfg(feature = "mint")]
        impl From<$mint_type> for $name {
            #[inline(always)]
            fn from(other: $mint_type) -> Self {
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
        impl From<$name> for $mint_type {
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

define_matrices! {
    (Mat2x2, ColumnMatrix2, f32, Vec2, 2 * 2, align: 8, size: 16, pad: 0, [0, 1]),
    (Mat2x3, ColumnMatrix2x3, f32, Vec2, 2 * 3, align: 8, size: 32, pad: 8, [0, 1, 2]),
    (Mat2x4, ColumnMatrix2x4, f32, Vec2, 2 * 4, align: 8, size: 32, pad: 0, [0, 1, 2, 3]),

    (Mat3x2, ColumnMatrix3x2, f32, Vec3, 3 * 2, align: 16, size: 32, pad: 4, [0, 1]),
    (Mat3x3, ColumnMatrix3, f32, Vec3, 3 * 3, align: 16, size: 48, pad: 4, [0, 1, 2]),
    (Mat3x4, ColumnMatrix3x4, f32, Vec3, 3 * 4, align: 16, size: 64, pad: 4, [0, 1, 2, 3]),

    (Mat4x2, ColumnMatrix4x2, f32, Vec4, 4 * 2, align: 16, size: 32, pad: 0, [0, 1]),
    (Mat4x3, ColumnMatrix4x3, f32, Vec4, 4 * 3, align: 16, size: 48, pad: 0, [0, 1, 2]),
    (Mat4x4, ColumnMatrix4, f32, Vec4, 4 * 4, align: 16, size: 64, pad: 0, [0, 1, 2, 3]),

    (DMat2x2, ColumnMatrix2, f64, DVec2, 2 * 2, align: 16, size: 32, pad: 0, [0, 1]),
    (DMat2x3, ColumnMatrix2x3, f64, DVec2, 2 * 3, align: 16, size: 48, pad: 0, [0, 1, 2]),
    (DMat2x4, ColumnMatrix2x4, f64, DVec2, 2 * 4, align: 16, size: 64, pad: 0, [0, 1, 2, 3]),

    (DMat3x2, ColumnMatrix3x2, f64, DVec3, 3 * 2, align: 32, size: 64, pad: 0, [0, 1]),
    (DMat3x3, ColumnMatrix3, f64, DVec3, 3 * 3, align: 32, size: 96, pad: 0, [0, 1, 2]),
    (DMat3x4, ColumnMatrix3x4, f64, DVec3, 3 * 4, align: 32, size: 128, pad: 0, [0, 1, 2, 3]),

    (DMat4x2, ColumnMatrix4x2, f64, DVec4, 4 * 2, align: 32, size: 64, pad: 0, [0, 1]),
    (DMat4x3, ColumnMatrix4x3, f64, DVec4, 4 * 3, align: 32, size: 96, pad: 0, [0, 1, 2]),
    (DMat4x4, ColumnMatrix4, f64, DVec4, 4 * 4, align: 32, size: 128, pad: 0, [0, 1, 2, 3]),
}

/// Matrix of f32s with 2 columns and 2 rows. Alignment 8, size 16.
pub type Mat2 = Mat2x2;
/// Matrix of f32s with 3 columns and 3 rows. Alignment 16, size 48.
pub type Mat3 = Mat3x3;
/// Matrix of f32s with 4 columns and 4 rows. Alignment 16, size 64.
pub type Mat4 = Mat4x4;
/// Matrix of f64s with 2 columns and 3 rows. Alignment 16, size 48.
pub type DMat2 = DMat2x2;
/// Matrix of f64s with 3 columns and 3 rows. Alignment 32, size 96.
pub type DMat3 = DMat3x3;
/// Matrix of f64s with 4 columns and 4 rows. Alignment 32, size 128.
pub type DMat4 = DMat4x4;

/// Pads an element to be in an array in a shader.
///
/// All elements in arrays need to be aligned to 16 bytes. This automatically aligns your types to 16 bytes.
///
/// This glsl:
///
/// ```glsl
/// struct FloatArray {
///     float array[45];
/// };
/// ```
///
/// turns into:
///
/// ```rust
/// #[repr(C)]
/// struct FloatArray {
///     array: [shader_types::ArrayMember<f32>; 45]
/// }
/// ```
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ArrayMember<T>(pub T);

#[cfg(feature = "bytemuck")]
unsafe impl<T: bytemuck::Zeroable> bytemuck::Zeroable for ArrayMember<T> {}
#[cfg(feature = "bytemuck")]
unsafe impl<T: bytemuck::Pod> bytemuck::Pod for ArrayMember<T> {}

/// Pads a structure for use with dynamic offsets in graphics apis.
///
/// All dynamic offsets need to be aligned to 256 bytes. This automatically aligns your types to 256s.
///
/// Given a shader of:
///
/// ```glsl
/// uniform Uniforms {
///     mat4 mvp;
///     mat4 mv;
/// };
/// ```
///
/// An array of rust structs can be made and used:
///
/// ```rust
/// use shader_types::{Mat4, DynamicOffsetMember};
/// # use std::mem::size_of;
///
/// // Implementations don't matter
/// fn generate_mvp(_: usize) -> [f32; 16] {
///     // ...
/// #     unsafe { std::mem::zeroed() }
/// }
/// fn generate_mv(_: usize) -> [f32; 16] {
///     // ...
/// #     unsafe { std::mem::zeroed() }
/// }
/// fn set_uniform_buffer(_: &[DynamicOffsetMember<Uniforms>]) {
///     // ...
/// }
/// fn bind_uniform_with_offset(_: usize) {
///     // ...
/// }
/// fn render_object(_: usize) {
///     // ...
/// }
///
/// #[repr(C)]
/// struct Uniforms {
///     mvp: Mat4,
///     mv: Mat4,
/// }
///
/// // Generate buffer
/// let mut vec: Vec<DynamicOffsetMember<Uniforms>> = Vec::new();
/// for obj_idx in 0..10 {
///     vec.push(DynamicOffsetMember(Uniforms {
///         mvp: Mat4::from(generate_mvp(obj_idx)),
///         mv: Mat4::from(generate_mv(obj_idx)),
///     }))
/// }
///
/// // Use Buffer
/// set_uniform_buffer(&vec);
/// for obj_idx in 0..10 {
///     let offset = obj_idx * size_of::<DynamicOffsetMember<Uniforms>>();
///     // Offset must be aligned by 256
///     assert_eq!(offset % 256, 0);
///     bind_uniform_with_offset(offset);
///     render_object(obj_idx);
/// }
/// ```
#[repr(C, align(256))]
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DynamicOffsetMember<T>(pub T);

#[cfg(feature = "bytemuck")]
unsafe impl<T: bytemuck::Zeroable> bytemuck::Zeroable for DynamicOffsetMember<T> {}
#[cfg(feature = "bytemuck")]
unsafe impl<T: bytemuck::Pod> bytemuck::Pod for DynamicOffsetMember<T> {}

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
