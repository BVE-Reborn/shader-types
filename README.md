# shader-types

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/BVE-Reborn/shader-types/CI)
[![Crates.io](https://img.shields.io/crates/v/shader-types)](https://crates.io/crates/shader-types)
[![Documentation](https://docs.rs/shader-types/badge.svg)](https://docs.rs/shader-types)
![License](https://img.shields.io/crates/l/shader-types)

Vector and Matrix types that are properly aligned for use in std140 uniforms.

All the types in this library have the same alignment and size as the equivilant glsl type in the
default mode (std140).

This fixes the padding within members of structs but padding between members needs to be minded.
The types in [`padding`](https://docs.rs/shader-types/*/shader_types/padding/index.html) are there to make this easier.

Vectors are constructable to/from an array of their underlying type. Matrices are constructable
to/from both 1d and 2d arrays as well as an array of the underlying _vector_ type. (eg. [`Mat2`](https://docs.rs/shader-types/*/shader_types/type.Mat2.html) can be
constructed from `[Vec2; 2]`)

## Example

For the following glsl:

```glsl
layout(set = 0, binding = 0) uniform Block {
    mat4 mvp;
    vec3 position;
    vec3 normal;
    vec2 uv;
    int constants[3];
};
```

This struct is rife with padding. However it's now easy to mind the padding:

```rust
use shader_types::{Vec2, Vec3, Mat4, ArrayMember};

// Definition
#[repr(C)]
#[derive(Copy, Clone)]
struct UniformBlock {
    mvp: Mat4, // 16 align + 64 size
    position: Vec3, // 16 align + 12 size
    normal: Vec3, // 16 align + 12 size
    uv: Vec2, // 8 align + 8 size
    constants: [ArrayMember<i32>; 3] // 3x 16 align + 4 size
}

fn generate_mvp() -> [f32; 16] {
    // ...
}

// Construction
let block = UniformBlock {
    // Anything that can be converted to a [f32; 16] or [[f32; 4]; 4] works
    mvp: Mat4::from(generate_mvp()),
    position: Vec3::new([0.0, 1.0, 2.0]), // `from` also works
    normal: Vec3::new([-2.0, 2.0, 3.0]),
    uv: Vec2::new([0.0, 1.0]),
    constants: [ArrayMember(0), ArrayMember(1), ArrayMember(2)]
};

// Supports bytemuck with the `bytemuck` feature
unsafe impl bytemuck::Zeroable for UniformBlock {}
unsafe impl bytemuck::Pod for UniformBlock {}

let block_u8: &[u8] = bytemuck::cast_slice(&[block]);
```

License: MIT OR Apache-2.0 OR Zlib
