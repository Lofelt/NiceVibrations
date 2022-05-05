# Core
The core consists of a library, Rust API and a C API. More precisely, it has the following crates:
- `lib` - contains the source code of the core library and a Rust API
- `api` - depends on `lib` and contains the C bindings to the Rust API. A C library and header are the output of this crate.`api` and `lib` use the `c-to-rust-and-back-with-data` code pattern. It is explained step-by-step in the [code-patterns repo](https://github.com/Lofelt/code-patterns/tree/master/c-to-rust-and-back-with-data).
- `datamodel` - contains the Lofelt Data model related functions, schema and versioning.
- `realtime-audio-to-haptics` - algorithms for generating haptic events from analyzing realtime audio. See also `offline-audio-to-haptics`
- `dsp` - Low-level DSP components used by the `audio-to-haptics` libraries.
- `utils` - contains utility functions and classes like `Error`, etc

