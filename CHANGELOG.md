# Changelog

All notable changes to this project will be documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to cargo's version of [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

- [Unreleased](#unreleased)
- [v0.2.0](#v020)
- [v0.1.0](#v010)
- [Diffs](#diffs)

## Unreleased

#### Added
- `From<mint::*>` impls for all types.
- `From<shader_types::*>` impls for mint types.
- `from_mint` functions for easy conversion from anything that is convertable to a mint type.
- `ArrayMember` and `DynamicOffsetMember` types.
- `#[no_std]` support though disabling the `std` feature.

## v0.2.0

Released 2020-07-16

#### Changes
- This makes using this crate _technically_ UB, but works completely correctly.
  The old version also had UB, but that one actually messed up your code.

## v0.1.0

Released 2020-07-12

#### Added
- All vector, matrix, and padding types.

## Diffs

- [Unreleased](https://github.com/BVE-Reborn/shader-types/compare/v0.2.0...HEAD)
- [v0.2.0](https://github.com/BVE-Reborn/shader-types/compare/v0.1.0...v0.2.0)
