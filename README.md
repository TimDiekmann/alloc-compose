Alloc-Compose
=============

[![Test Status](https://github.com/TimDiekmann/alloc-compose/workflows/Test/badge.svg?event=push&branch=master)](https://github.com/TimDiekmann/alloc-compose/actions?query=workflow%3ATest+event%3Apush+branch%3Amaster)
[![Lint Status](https://github.com/TimDiekmann/alloc-compose/workflows/Lint/badge.svg?event=push&branch=master)](https://github.com/TimDiekmann/alloc-compose/actions?query=workflow%3ALint+event%3Apush+branch%3Amaster)
[![Docs master](https://img.shields.io/static/v1?label=docs&message=master&color=5479ab)](https://timdiekmann.github.io/alloc-compose/alloc_compose/index.html)
[![Docs.rs](https://docs.rs/alloc-compose/badge.svg)](https://docs.rs/alloc-compose)
[![Crates.io](https://img.shields.io/crates/v/alloc-compose)](https://crates.io/crates/alloc-compose)
![Crates.io](https://img.shields.io/crates/l/alloc-compose)

Composable allocator structures for plugging together more powerful allocators.

`alloc-compose` currently uses the [`alloc-wg`] crate as backend. As soon as all features
has landed upstream, this dependency will be dropped. Until `AllocRef` has been
stabilized, this crate requires a nightly compiler.

[`alloc-wg`]: https://crates.io/crates/alloc-wg

The design of composable allocators is inspired by
[`std::allocator` Is to Allocation what `std::vector` Is to Vexation][vid] by Andrei
Alexandrescu and the [Phobos Standard Library][phobos] of the [D Programming Language][D].

[vid]: https://www.youtube.com/watch?v=LIb3L4vKZ7U
[phobos]: https://github.com/dlang/phobos
[D]: https://dlang.org/

License
-------
Alloc-Compose is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/TimDiekmann/alloc-compose/blob/master/LICENSE-APACHE) and [LICENSE-MIT](https://github.com/TimDiekmann/alloc-compose/blob/master/LICENSE-MIT) for details.
