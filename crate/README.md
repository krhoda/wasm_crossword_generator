# WASM Library Packaging Example -- The Rust Lib
This directory is inner crate used by the Rollup JS lib in which this directory sits. It's primarily used for adding functionality to the surrounding library, but it is also published seperately on it's own terms to allow other WASM libraries to re-export it. Such a library can be found at `../../examples/wasm-dep-lib`. This approach is much more efficient and easier than requiring users to bring in two WASM blobs.

This library is as simple of a toy demo as can be while still demonstrating all the needed configuration for the ideal WASM library build process.
