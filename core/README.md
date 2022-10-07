# Core
The core consists of a library, Rust API and a C API. More precisely, it has the following crates:
- `lib` - contains the source code of the core library and a Rust API
- `api` - depends on `lib` and contains the C bindings to the Rust API. A C library and header are the output of this crate.
- `datamodel` - contains the Lofelt Data model related functions, schema and versioning.
- `utils` - contains utility functions and classes like `Error`, etc
