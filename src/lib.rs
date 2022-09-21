// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

// https://github.com/unicode-org/icu4x/blob/main/docs/process/boilerplate.md#library-annotations
#![no_std]
#![cfg_attr(
    not(test),
    deny(
        clippy::indexing_slicing,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        // Exhaustiveness and Debug is not required for Diplomat types
    )
)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::needless_lifetimes)]

//! This module contains the source of truth for the [Diplomat](https://github.com/rust-diplomat/diplomat)-generated
//! FFI bindings. This generates the C, C++ and Wasm bindings. This module also contains the C
//! FFI for ICU4X.
//!
//! To re-generate the bindings run:
//!
//! ```sh
//! cargo make diplomat-install
//! cargo make diplomat-gen
//! ```

extern crate alloc;

#[macro_use]
mod utils;

//pub mod data_struct;
pub mod errors;
pub mod fallbacker;
pub mod locale;
//pub mod locid_transform;
//pub mod logging;
//pub mod properties_maps;
//pub mod properties_sets;
pub mod provider;
//pub mod script;
//pub mod segmenter_grapheme;
pub mod segmenter_line;
//pub mod segmenter_sentence;
//#pub mod segmenter_word;
