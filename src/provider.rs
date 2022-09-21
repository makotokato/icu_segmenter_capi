// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#[allow(unused_imports)] // feature-specific
use alloc::boxed::Box;
use icu_provider::prelude::*;
#[allow(unused_imports)] // feature-specific
use icu_provider::RcWrapBounds;
use icu_provider_adapters::empty::EmptyDataProvider;
#[allow(unused_imports)] // feature-specific
use yoke::{trait_hack::YokeTraitHack, Yokeable};
#[allow(unused_imports)] // feature-specific
use zerofrom::ZeroFrom;

pub enum ICU4XDataProviderInner {
    Empty,
    Buffer(Box<dyn BufferProvider + 'static>),
}

impl Default for ICU4XDataProviderInner {
    fn default() -> Self {
        Self::Empty
    }
}

#[diplomat::bridge]
pub mod ffi {
    use super::ICU4XDataProviderInner;
    use crate::errors::ffi::ICU4XError;
    use crate::fallbacker::ffi::ICU4XLocaleFallbacker;
    use alloc::boxed::Box;
    use diplomat_runtime::DiplomatResult;
    use icu_provider_adapters::fallback::LocaleFallbackProvider;
    use icu_provider_adapters::fork::predicates::MissingLocalePredicate;

    #[diplomat::opaque]
    /// An ICU4X data provider, capable of loading ICU4X data keys from some source.
    #[diplomat::rust_link(icu_provider, Mod)]
    pub struct ICU4XDataProvider(pub ICU4XDataProviderInner);

    /// A result type for `ICU4XDataProvider::create`.
    pub struct ICU4XCreateDataProviderResult {
        /// Will be `None` if `success` is `false`, do not use in that case.
        pub provider: Option<Box<ICU4XDataProvider>>,
        // May potentially add a better error type in the future
        pub success: bool,
    }

    fn convert_buffer_provider<D: icu_provider::BufferProvider + 'static>(
        x: D,
    ) -> Box<ICU4XDataProvider> {
        Box::new(ICU4XDataProvider(
            super::ICU4XDataProviderInner::from_buffer_provider(x),
        ))
    }

    impl ICU4XDataProvider {
        /// Constructs a `BlobDataProvider` and returns it as an [`ICU4XDataProvider`].
        #[diplomat::rust_link(icu_provider_blob::BlobDataProvider, Struct)]
        #[allow(unused_variables)] // conditional on features
        pub fn create_from_byte_slice(
            blob: &[u8],
        ) -> DiplomatResult<Box<ICU4XDataProvider>, ICU4XError> {
            icu_provider_blob::BlobDataProvider::try_new_from_blob(blob)
                .map_err(Into::into)
                .map(convert_buffer_provider)
                .into()
        }

        /// Constructs an empty `StaticDataProvider` and returns it as an [`ICU4XDataProvider`].
        #[diplomat::rust_link(icu_provider_adapters::empty::EmptyDataProvider, Struct)]
        #[diplomat::rust_link(
            icu_provider_adapters::empty::EmptyDataProvider::new,
            FnInStruct,
            hidden
        )]
        pub fn create_empty() -> Box<ICU4XDataProvider> {
            Box::new(ICU4XDataProvider(ICU4XDataProviderInner::Empty))
        }

        /// Creates a provider that tries the current provider and then, if the current provider
        /// doesn't support the data key, another provider `other`.
        ///
        /// This takes ownership of the `other` provider, leaving an empty provider in its place.
        ///
        /// The providers must be the same type (Any or Buffer). This condition is satisfied if
        /// both providers originate from the same constructor, such as `create_from_byte_slice`
        /// or `create_fs`. If the condition is not upheld, a runtime error occurs.
        #[diplomat::rust_link(icu_provider_adapters::fork::ForkByKeyProvider, Typedef)]
        #[diplomat::rust_link(
            icu_provider_adapters::fork::predicates::MissingDataKeyPredicate,
            Struct,
            hidden
        )]
        pub fn fork_by_key(
            &mut self,
            other: &mut ICU4XDataProvider,
        ) -> DiplomatResult<(), ICU4XError> {
            let a = core::mem::take(&mut self.0);
            let b = core::mem::take(&mut other.0);
            match (a, b) {
                (ICU4XDataProviderInner::Buffer(a), ICU4XDataProviderInner::Buffer(b)) => {
                    self.0 = ICU4XDataProviderInner::Buffer(Box::from(
                        icu_provider_adapters::fork::ForkByKeyProvider::new(a, b),
                    ));
                    Ok(())
                }
                _ => {
                    let e = ICU4XError::DataMismatchedAnyBufferError;
                    crate::errors::log_conversion(
                        &"fork_by_key must be passed the same type of provider (Any or Buffer)",
                        e,
                    );
                    Err(e)
                }
            }
            .into()
        }

        /// Same as `fork_by_key` but forks by locale instead of key.
        #[diplomat::rust_link(
            icu_provider_adapters::fork::predicates::MissingLocalePredicate,
            Struct
        )]
        pub fn fork_by_locale(
            &mut self,
            other: &mut ICU4XDataProvider,
        ) -> DiplomatResult<(), ICU4XError> {
            let a = core::mem::take(&mut self.0);
            let b = core::mem::take(&mut other.0);
            match (a, b) {
                (ICU4XDataProviderInner::Buffer(a), ICU4XDataProviderInner::Buffer(b)) => {
                    self.0 = ICU4XDataProviderInner::Buffer(Box::from(
                        icu_provider_adapters::fork::ForkByErrorProvider::new_with_predicate(
                            a,
                            b,
                            MissingLocalePredicate,
                        ),
                    ));
                    Ok(())
                }
                _ => {
                    let e = ICU4XError::DataMismatchedAnyBufferError;
                    crate::errors::log_conversion(
                        &"fork_by_locale must be passed the same type of provider (Any or Buffer)",
                        e,
                    );
                    Err(e)
                }
            }
            .into()
        }

        /// Enables locale fallbacking for data requests made to this provider.
        ///
        /// Note that the test provider (from `create_test`) already has fallbacking enabled.
        #[diplomat::rust_link(
            icu_provider_adapters::fallback::LocaleFallbackProvider::try_new_unstable,
            FnInStruct
        )]
        #[diplomat::rust_link(
            icu_provider_adapters::fallback::LocaleFallbackProvider,
            Struct,
            compact
        )]
        pub fn enable_locale_fallback(&mut self) -> DiplomatResult<(), ICU4XError> {
            match core::mem::take(&mut self.0) {
                ICU4XDataProviderInner::Empty => Err(icu_provider::DataErrorKind::MissingDataKey
                    .into_error()
                    .into()),
                ICU4XDataProviderInner::Buffer(inner) => {
                    match LocaleFallbackProvider::try_new_with_buffer_provider(inner) {
                        Ok(x) => {
                            self.0 = ICU4XDataProviderInner::Buffer(Box::new(x));
                            Ok(())
                        }
                        Err(e) => Err(e.into()),
                    }
                }
            }
            .into()
        }

        #[diplomat::rust_link(
            icu_provider_adapters::fallback::LocaleFallbackProvider::new_with_fallbacker,
            FnInStruct
        )]
        #[diplomat::rust_link(
            icu_provider_adapters::fallback::LocaleFallbackProvider,
            Struct,
            compact
        )]
        pub fn enable_locale_fallback_with(
            &mut self,
            fallbacker: &ICU4XLocaleFallbacker,
        ) -> DiplomatResult<(), ICU4XError> {
            match core::mem::take(&mut self.0) {
                ICU4XDataProviderInner::Empty => Err(icu_provider::DataErrorKind::MissingDataKey
                    .into_error()
                    .into()),
                ICU4XDataProviderInner::Buffer(inner) => {
                    self.0 = ICU4XDataProviderInner::Buffer(Box::new(
                        LocaleFallbackProvider::new_with_fallbacker(inner, fallbacker.0.clone()),
                    ));
                    Ok(())
                }
            }
            .into()
        }
    }
}

impl<M> DataProvider<M> for ICU4XDataProviderInner
where
    M: KeyedDataMarker + 'static,
    // Actual bound:
    //     for<'de> <M::Yokeable as Yokeable<'de>>::Output: Deserialize<'de>,
    // Necessary workaround bound (see `yoke::trait_hack` docs):
    for<'de> YokeTraitHack<<M::Yokeable as Yokeable<'de>>::Output>: serde::Deserialize<'de>,
{
    fn load(&self, req: DataRequest) -> Result<DataResponse<M>, DataError> {
        match self {
            ICU4XDataProviderInner::Empty => EmptyDataProvider::new().load(req),
            ICU4XDataProviderInner::Buffer(buffer_provider) => {
                buffer_provider.as_deserializing().load(req)
            }
        }
    }
}

impl ICU4XDataProviderInner {
    fn from_buffer_provider(buffer_provider: impl BufferProvider + 'static) -> Self {
        Self::Buffer(Box::new(buffer_provider))
    }
}
