## [v0.5](https://timdiekmann.github.io/alloc-compose/alloc_compose/index.html) (Unreleased)

**Breaking Changes:** 
- Add `AllocAll` trait and move some methods from `Region` into that trait
- Change `Region` to require `[MaybeUninit<u8>]` rather than `[u8]`
- Remove `MemoryMarker`

## [v0.4](https://docs.rs/alloc-compose/0.4)

- **Breaking Change** Using unified naming scheme
- **Breaking Change** Change `CallbackRef` to listen on `before_` and `after_` events
- Greatly improve documentation of `Affix`

### [v0.3.1](https://docs.rs/alloc-compose/0.3)

- Add more documentation
- Add more tests

## [v0.3.0](https://docs.rs/alloc-compose/0.3)

- **Breaking Change** Use `const_generics` in `SegregateAlloc`
- Add `AffixAlloc`, `ChunkAlloc`, and `MemoryMarker`
- Add more tests

## [v0.2](https://docs.rs/alloc-compose/0.2)

- **Breaking Change** Use `core::alloc` instead of `alloc_wg`
- Add `Region`, `CallbackRef`, `Proxy`, and `stats`
- Add more tests

## [v0.1](https://docs.rs/alloc-compose/0.1)

- Initial release: `Owns`, `NullAlloc`, `FallbackAlloc`, and `SegregateAlloc`
