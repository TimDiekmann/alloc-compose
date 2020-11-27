[![Test Status](https://github.com/TimDiekmann/alloc-compose/workflows/Test/badge.svg?event=push&branch=master)](https://github.com/TimDiekmann/alloc-compose/actions?query=workflow%3ATest+event%3Apush+branch%3Amaster)
[![Coverage Status](https://codecov.io/gh/TimDiekmann/alloc-compose/branch/master/graph/badge.svg)](https://codecov.io/gh/TimDiekmann/alloc-compose)
[![Docs master](https://img.shields.io/static/v1?label=docs&message=master&color=5479ab)](https://timdiekmann.github.io/alloc-compose/alloc_compose/index.html)
[![Docs.rs](https://docs.rs/alloc-compose/badge.svg)](https://docs.rs/alloc-compose)
[![Crates.io](https://img.shields.io/crates/v/alloc-compose)](https://crates.io/crates/alloc-compose)
![Crates.io](https://img.shields.io/crates/l/alloc-compose)

---

Important note
--------------

Due to some changes to `AllocRef` it was hard to keep this crate updated. I'll readd the functionality from v0.5.0 from time to time. Most things have to be refactored as `AllocRef`s reallocation methods now takes two layouts.

The most interesting part as of now is probably `Region` and its variants.
In future version, composable blocks like `AffixAllocator` or `Proxy` will be added.

---

Composable allocator structures for plugging together more powerful allocators.

`alloc-compose` relies on [`AllocRef`] as allocator trait. Until `AllocRef` has been stabilized, this crate requires a nightly compiler.


The design of composable allocators is inspired by
[`std::allocator` Is to Allocation what `std::vector` Is to Vexation][vid] by Andrei
Alexandrescu and the [Phobos Standard Library][phobos] of the [D Programming Language][D].

[`AllocRef`]: https://doc.rust-lang.org/nightly/core/alloc/trait.AllocRef.html
[vid]: https://www.youtube.com/watch?v=LIb3L4vKZ7U
[phobos]: https://github.com/dlang/phobos
[D]: https://dlang.org/

License
-------

Alloc-Compose is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/TimDiekmann/alloc-compose/blob/master/LICENSE-APACHE) and [LICENSE-MIT](https://github.com/TimDiekmann/alloc-compose/blob/master/LICENSE-MIT) for details.
