// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use self::ffi::ICU4XError;
use core::fmt;
use icu_locid::ParserError;
use icu_provider::{DataError, DataErrorKind};
use icu_segmenter::SegmenterError;
use tinystr::TinyStrError;

#[diplomat::bridge]
pub mod ffi {
    use alloc::boxed::Box;

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(C)]
    /// A common enum for errors that ICU4X may return, organized by API
    ///
    /// The error names are stable and can be checked against as strings in the JS API
    #[diplomat::rust_link(icu::locid::ParserError, Enum, compact)]
    #[diplomat::rust_link(icu::provider::DataError, Struct, compact)]
    #[diplomat::rust_link(icu::provider::DataErrorKind, Enum, compact)]
    #[diplomat::rust_link(icu::segmenter::SegmenterError, Enum, compact)]
    pub enum ICU4XError {
        // general errors
        /// The error is not currently categorized as ICU4XError.
        /// Please file a bug
        UnknownError = 0x00,
        /// An error arising from writing to a string
        /// Typically found when not enough space is allocated
        /// Most APIs that return a string may return this error
        WriteableError = 0x01,
        // Some input was out of bounds
        OutOfBoundsError = 0x02,

        // general data errors
        // See DataError
        DataMissingDataKeyError = 0x1_00,
        DataMissingVariantError = 0x1_01,
        DataMissingLocaleError = 0x1_02,
        DataNeedsVariantError = 0x1_03,
        DataNeedsLocaleError = 0x1_04,
        DataExtraneousLocaleError = 0x1_05,
        DataFilteredResourceError = 0x1_06,
        DataMismatchedTypeError = 0x1_07,
        DataMissingPayloadError = 0x1_08,
        DataInvalidStateError = 0x1_09,
        DataCustomError = 0x1_0A,
        DataIoError = 0x1_0B,
        DataUnavailableBufferFormatError = 0x1_0C,
        DataMismatchedAnyBufferError = 0x1_0D,

        // locale errors
        /// The subtag being requested was not set
        LocaleUndefinedSubtagError = 0x2_00,
        /// The locale or subtag string failed to parse
        LocaleParserLanguageError = 0x2_01,
        LocaleParserSubtagError = 0x2_02,
        LocaleParserExtensionError = 0x2_03,

        // data struct errors
        /// Attempted to construct an invalid data struct
        DataStructValidityError = 0x3_00,

        // tinystr errors
        TinyStrTooLargeError = 0x9_00,
        TinyStrContainsNullError = 0x9_01,
        TinyStrNonAsciiError = 0x9_02,
    }
}

impl ICU4XError {
    #[inline]
    pub(crate) fn log_original<T: core::fmt::Display + ?Sized>(self, _e: &T) -> Self {
        self
    }
}

impl From<fmt::Error> for ICU4XError {
    fn from(e: fmt::Error) -> Self {
        ICU4XError::WriteableError.log_original(&e)
    }
}

impl From<DataError> for ICU4XError {
    fn from(e: DataError) -> Self {
        match e.kind {
            DataErrorKind::MissingDataKey => ICU4XError::DataMissingDataKeyError,
            DataErrorKind::MissingLocale => ICU4XError::DataMissingLocaleError,
            DataErrorKind::NeedsLocale => ICU4XError::DataNeedsLocaleError,
            DataErrorKind::ExtraneousLocale => ICU4XError::DataExtraneousLocaleError,
            DataErrorKind::FilteredResource => ICU4XError::DataFilteredResourceError,
            DataErrorKind::MismatchedType(..) => ICU4XError::DataMismatchedTypeError,
            DataErrorKind::MissingPayload => ICU4XError::DataMissingPayloadError,
            DataErrorKind::InvalidState => ICU4XError::DataInvalidStateError,
            DataErrorKind::Custom => ICU4XError::DataCustomError,
            #[cfg(all(
                feature = "provider_fs",
                not(any(target_arch = "wasm32", target_os = "none"))
            ))]
            DataErrorKind::Io(..) => ICU4XError::DataIoError,
            // datagen only
            // DataErrorKind::MissingSourceData(..) => ..,
            DataErrorKind::UnavailableBufferFormat(..) => {
                ICU4XError::DataUnavailableBufferFormatError
            }
            _ => ICU4XError::UnknownError,
        }
        .log_original(&e)
    }
}

impl From<SegmenterError> for ICU4XError {
    fn from(e: SegmenterError) -> Self {
        match e {
            SegmenterError::Data(e) => e.into(),
            _ => ICU4XError::UnknownError,
        }
        .log_original(&e)
    }
}

impl From<ParserError> for ICU4XError {
    fn from(e: ParserError) -> Self {
        match e {
            ParserError::InvalidLanguage => ICU4XError::LocaleParserLanguageError,
            ParserError::InvalidSubtag => ICU4XError::LocaleParserSubtagError,
            ParserError::InvalidExtension => ICU4XError::LocaleParserExtensionError,
            _ => ICU4XError::UnknownError,
        }
        .log_original(&e)
    }
}

impl From<TinyStrError> for ICU4XError {
    fn from(e: TinyStrError) -> Self {
        match e {
            TinyStrError::TooLarge { .. } => ICU4XError::TinyStrTooLargeError,
            TinyStrError::ContainsNull => ICU4XError::TinyStrContainsNullError,
            TinyStrError::NonAscii => ICU4XError::TinyStrNonAsciiError,
            _ => ICU4XError::UnknownError,
        }
        .log_original(&e)
    }
}
