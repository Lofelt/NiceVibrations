Crate for a playing back pre-authored clips with various different implementations.

All implementations for playing back pre-authored clips are based on the
`PreAuthoredClipPlayback` trait.

`PreAuthoredClipPlayback` implementations:
- `android::Player`, only included when compiling for Android as the target OS
- `null::Player`, a dummy player compiled for all target OSes
- `streaming::Player`, streams clip breakpoints to callbacks
