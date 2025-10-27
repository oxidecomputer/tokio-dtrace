// Copyright 2025 Oxide Computer Company

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]
//!
//! ## Usage
//!
//! ### Enabling `tokio_unstable` Features
//!
//! `tokio-dtrace` requires Tokio's [unstable features] in order to use runtime
//! hooks. These are features of Tokio that do not yet have stable APIs, and may
//! change in 1.x releases. Unlike other optional features, Tokio requires that
//! only the top-level binary workspace may opt in to these features (i.e., they
//! may not be enabled by library dependencies).This means that the unstable
//! features are enabled using a `RUSTFLAGS` config, rather than a Cargo
//! feature.
//!
//! The simplest way to enable Tokio's unstable features is to add the
//! following to your workspace's `.cargo/config.toml` file:
//!
//! ```toml
//! [build]
//! rustflags = ["--cfg", "tokio_unstable"]
//! ```
//!
//! <div class="warning">
//! The <code>[build]</code> section does <strong>not</strong> go in a
//! <code>Cargo.toml</code> file. Instead it must be placed in the Cargo config
//! file <code>.cargo/config.toml</code>.
//! </div>
//!
//! For more details, see [Tokio's documentation on unstable
//! features][unstable features].
//!
//! ### Adding Runtime Hooks
//!
//! Once Tokio's unstable features are enabled, `tokio-dtrace`'s runtime hooks
//! must be added to the application's Tokio runtime. The easiest way to do this
//! is to pass a mutable reference to the application's
//! [`tokio::runtime::Builder`] to the
//! [`tokio_dtrace::register_hooks`](crate::register_hooks) function.
//!
//! For example:
//! ```
//! // Construct a new Tokio runtime builder.
//! let mut builder = tokio::runtime::Builder::new_multi_thread();
//!
//! // Attempt to register `tokio-dtrace` hooks with the runtime.
//! let rt = tokio_dtrace::register_hooks(&mut builder)
//!     // `register_hooks` returns an error if DTrace probes could
//!     // not be enabled.
//!     .unwrap()
//!     // Enable other Tokio runtime features, and configure other builder
//!     // settings as needed...
//!     .enable_all()
//!     .build()
//!     .unwrap();
//!
//! // ... Use the configured runtime in your application ...
//! # drop(rt);
//! ```
//!
//! Note that, because `tokio-dtrace` requires the use of the
//! [`tokio::runtime::Builder`] to add hooks to the runtime, it is not possible
//! to use `tokio-dtrace` with the [`tokio::main`] attribute macro.
//! Fortunately, code using `#[tokio::main]` can be transformed to code using
//! the runtime builder fairly simply.
//!
//! For example, this:
//!
//! ```rust
//! # async fn do_stuff() {}
//! #[tokio::main(flavor = "multi_thread", worker_threads = 10)]
//! async fn main() {
//!     do_stuff().await;
//! }
//! ```
//!
//! becomes this:
//!
//!```rust
//! # async fn do_stuff() {}
//! fn main() {
//!     let mut builder = tokio::runtime::Builder::new_multi_thread();
//!     tokio_dtrace::register_hooks(&mut builder).unwrap();
//!     let rt = builder.enable_all().worker_threads(10).build().unwrap();
//!     rt.block_on(async {
//!         tokio::spawn(async {
//!             do_stuff().await;
//!         })
//!         .await
//!         .unwrap()
//!     });
//! }
//! ```
//!
//! See the documentation for
//! [`tokio_dtrace::register_hooks`](crate::register_hooks) for more
//! information.
//!
//! Note the `register_hooks` function sets a function to be called in all of
//! Tokio's unstable runtime hook callbacks. If additional code must also be
//! called in one or more of these hooks, refer to the documentation for the
//! [`hooks`] module for more complex uses.
//!
//! [unstable features]: https://docs.rs/tokio/latest/tokio/#unstable-features
//! [`tokio::main`]: https://docs.rs/tokio/latest/tokio/attr.main.html
//!

/// Registers `tokio-dtrace`s probe hooks with the provided
/// [`tokio::runtime::Builder`].
///
/// This function configures the runtime to emit DTrace probes. In addition,
/// it also registers the USDT probes provided by this crate with DTrace, and
/// calls the [`check_casts`] function to ensure that type layout in Tokio has
/// not changed in a way that would render unsafe casts used by `tokio-dtrace`
/// unsound.
///
/// Note that this sets a function to be called in all of Tokio's unstable
/// runtime hook callbacks. If additional code must also be called in one or
/// more of these hooks, refer to the documentation for the [`hooks`] module for
/// more complex uses.
///
/// ## Errors
///
/// This function returns [an error](RegistrationError) in the following
/// conditions:
///
/// - [`RegistrationError::UnstableFeaturesRequired`] if Tokio's
///   [unstable features](crate#enabling-tokio_unstable-features) are not
///   enabled at compile time.
/// - [`RegistrationError::DTrace`] if the [`usdt`] crate returns an error
///   when registering probes with DTrace.
/// - [`RegistrationError::InvalidCasts`] if a call to [`check_casts`] fails,
///   which would indicate that type layout in Tokio has changed in a way that
///   would render unsafe casts used by `tokio-dtrace` unsound.
///
/// ## Examples
///
/// Basic usage:
///
/// ```
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Construct a new Tokio runtime builder.
///     let mut builder = tokio::runtime::Builder::new_multi_thread();
///     // Attempt to register `tokio-dtrace` hooks with the runtime.
///     tokio_dtrace::register_hooks(&mut builder)?;
///
///     let rt = builder
///         // Enable other Tokio runtime features, and configure other builder
///         // settings as needed...
///         .enable_all()
///         // Finish building the runtime.
///         .build()?;
///
///     // Use the constructed runtime to run the application.
///     rt.block_on(async {
///         // Your application code here
///     });
///
///     Ok(())
/// }
/// ```
///
/// In some cases, such as when the application may run in environments where
/// DTrace may not be available, it may be desirable to allow `tokio-dtrace`
/// registration to fail, rather than exiting. For example:
///
/// ```
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut builder = tokio::runtime::Builder::new_multi_thread();
///
///     if let Err(e) = tokio_dtrace::register_hooks(&mut builder) {
///         // If registering tokio-dtrace hooks fails, print a warning
///         // message, but continue running the application.
///         eprintln!("WARNING: could not register Tokio DTrace probes: {e}");
///     }
///
///     let rt = builder
///         .enable_all()
///         .build()?;
///
///     rt.block_on(async {
///         // Your application code here
///     });
///
///     Ok(())
/// }
/// ```
pub fn register_hooks(
    builder: &mut tokio::runtime::Builder,
) -> Result<&mut tokio::runtime::Builder, RegistrationError> {
    #[cfg(tokio_unstable)]
    {
        usdt::register_probes()?;
        let builder = builder
            .on_thread_start(hooks::on_thread_start)
            .on_thread_park(hooks::on_thread_park)
            .on_thread_unpark(hooks::on_thread_unpark)
            .on_thread_stop(hooks::on_thread_stop)
            .on_task_spawn(hooks::on_task_spawn)
            .on_before_task_poll(hooks::on_before_task_poll)
            .on_after_task_poll(hooks::on_after_task_poll)
            .on_task_terminate(hooks::on_task_terminate);
        Ok(builder)
    }
    #[cfg(not(tokio_unstable))]
    {
        Err(RegistrationError::UnstableFeaturesRequired)
    }
}

/// Errors returned by [`register_hooks`].
#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    /// `tokio-dtrace` hooks cannot be registered, as [Tokio's unstable
    /// features][unstable] are not enabled.
    ///
    /// [unstable]: crate#enabling-tokio_unstable-features
    #[error("tokio-dtrace requires `RUSTFLAGS=\"--cfg tokio_unstable\"`")]
    UnstableFeaturesRequired,

    /// Probes could not be registered with DTrace.
    #[error(transparent)]
    DTrace(#[from] usdt::Error),
}

/// Tokio runtime hooks for DTrace probes.
///
/// This module contains functions that are called by the Tokio runtime when
/// events occur in order to emit DTrace probes for those events. They are
/// intended to be passed to a [`tokio::runtime::Builder`]'s methods that set
/// the hook functions called by the runtime being constructed:
///
/// - [`tokio::runtime::Builder::on_task_spawn`]
/// - [`tokio::runtime::Builder::on_before_task_poll`]
/// - [`tokio::runtime::Builder::on_after_task_poll`]
/// - [`tokio::runtime::Builder::on_task_terminate`]
/// - [`tokio::runtime::Builder::on_thread_start`]
/// - [`tokio::runtime::Builder::on_thread_stop`]
/// - [`tokio::runtime::Builder::on_thread_park`]
/// - [`tokio::runtime::Builder::on_thread_unpark`]
///
/// Typically, users of `tokio-dtrace` need not interact with these functions
/// directly: the [`register_hooks`] function can be used to register all
/// runtime hooks in a single function call. However, the Tokio runtime only
/// allows a *single* function to be set for each of these hooks. Therefore, if
/// the application needs to run other code in addition to `tokio-dtrace`'s
/// probes in one or more runtime hooks, it is necessary to write a wrapper
/// function that calls the `tokio-dtrace` hooks as well as any other code that
/// must run in the hook callback.
///
/// For instance, if I have some function `other_on_task_spawn_thing()` that I
/// would like to run in the
/// [`on_task_spawn`](tokio::runtime::Builder::on_task_spawn) hook, I can write
/// a wrapper function like this:
///
/// ```rust
/// use tokio::runtime::TaskMeta;
/// # fn other_on_task_spawn_thing(meta: &TaskMeta<'_>) {};
///
/// fn on_task_spawn(meta: &TaskMeta<'_>) {
///     // Call the tokio-dtrace hook.
///     tokio_dtrace::hooks::on_task_spawn(meta);
///     // Call the other function that should run in the `on_task_spawn` hook.
///     other_on_task_spawn_thing(meta);
/// }
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Check that tokio-dtrace's casts are valid, and register DTrace probes.
///     tokio_dtrace::check_casts()?;
///     usdt::register_probes()?;
///
///     // Construct a new Tokio runtime builder.
///     let rt = tokio::runtime::Builder::new_multi_thread()
///         // Register the `on_task_spawn` hook defined above.
///         .on_task_spawn(on_task_spawn)
///         // Register `tokio-dtrace`'s other hooks.
///         .on_before_task_poll(tokio_dtrace::hooks::on_before_task_poll)
///         .on_after_task_poll(tokio_dtrace::hooks::on_after_task_poll)
///         .on_task_terminate(tokio_dtrace::hooks::on_task_terminate)
///         .on_thread_start(tokio_dtrace::hooks::on_thread_start)
///         .on_thread_stop(tokio_dtrace::hooks::on_thread_stop)
///         .on_thread_park(tokio_dtrace::hooks::on_thread_park)
///         .on_thread_unpark(tokio_dtrace::hooks::on_thread_unpark)
///         .on_thread_stop(tokio_dtrace::hooks::on_thread_stop)
///         // Enable other Tokio runtime features, and configure other builder
///         // settings as needed...
///         .enable_all()
///         // Finish building the runtime.
///         .build()?;
///
///     rt.block_on(async {
///         // Your application code here
///     });
///
///     Ok(())
/// }
/// ```
///
/// If only a small number of hooks need to call additional functions in this
/// manner, it is also possible to use the [`tokio_dtrace::register_hooks`]
/// function, and then override a subset of the hooks with user-defined
/// functions. Note that the calls to [`tokio_dtrace::register_hooks`] should be
/// made *before* overriding any hooks with user-defined code, as calling the
/// runtime builder method *overrides* any previously registered function for
/// that hook.
///
/// In that case, the previous example reduces to:
///
/// ```rust
/// use tokio::runtime::TaskMeta;
/// # fn other_on_task_spawn_thing(meta: &TaskMeta<'_>) {};
///
/// fn on_task_spawn(meta: &TaskMeta<'_>) {
///     // Call the tokio-dtrace hook.
///     tokio_dtrace::hooks::on_task_spawn(meta);
///     // Call the other function that should run in the `on_task_spawn` hook.
///     other_on_task_spawn_thing(meta);
/// }
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut builder  = tokio::runtime::Builder::new_multi_thread();
///     // Register the default `tokio-dtrace` hooks.
///     //
///     // Since `register_hooks` also checks for cast validitiy and registers
///     // DTrace probes, it is not necessary to call those functions
///     // separately this time.
///     let rt = tokio_dtrace::register_hooks(&mut builder)?
///         // Override *just* the `on_task_spawn` hook.
///         .on_task_spawn(on_task_spawn)
///         // Enable other Tokio runtime features, and configure other builder
///         // settings as needed...
///         .enable_all()
///         // Finish building the runtime.
///         .build()?;
///
///     rt.block_on(async {
///         // Your application code here
///     });
///
///     Ok(())
/// }
/// ```
///
/// [`tokio_dtrace::register_hooks`]: crate::register_hooks
#[cfg(tokio_unstable)]
pub mod hooks {
    use super::probes;
    use tokio::runtime::TaskMeta;

    /// Hook function to be used in [`tokio::runtime::Builder::on_task_spawn`].
    pub fn on_task_spawn(meta: &TaskMeta<'_>) {
        probes::task__spawn!(|| unpack_meta(meta));
    }

    /// Hook function to be used in [`tokio::runtime::Builder::on_before_task_poll`].
    pub fn on_before_task_poll(meta: &TaskMeta<'_>) {
        probes::task__poll__start!(|| unpack_meta(meta));
    }

    /// Hook function to be used in [`tokio::runtime::Builder::on_after_task_poll`].
    pub fn on_after_task_poll(meta: &TaskMeta<'_>) {
        probes::task__poll__end!(|| unpack_meta(meta));
    }

    /// Hook function to be used in [`tokio::runtime::Builder::on_task_terminate`].
    pub fn on_task_terminate(meta: &TaskMeta<'_>) {
        probes::task__terminate!(|| unpack_meta(meta));
    }

    /// Hook function to be used in [`tokio::runtime::Builder::on_thread_start`].
    pub fn on_thread_start() {
        probes::worker__thread__start!(|| ());
    }

    /// Hook function to be used in [`tokio::runtime::Builder::on_thread_stop`].
    pub fn on_thread_stop() {
        probes::worker__thread__stop!(|| ());
    }

    /// Hook function to be used in [`tokio::runtime::Builder::on_thread_park`].
    pub fn on_thread_park() {
        probes::worker__thread__park!(|| ());
    }

    /// Hook function to be used in [`tokio::runtime::Builder::on_thread_unpark`].
    pub fn on_thread_unpark() {
        probes::worker__thread__unpark!(|| ());
    }

    #[inline]
    fn unpack_meta(meta: &TaskMeta<'_>) -> (u64, String, u32, u32) {
        let id = id_to_u64(meta.id());
        let location = meta.spawned_at();
        let file = location.file().to_string();
        let line = location.line();
        let col = location.column();
        (id, file, line, col)
    }

    #[inline]
    fn id_to_u64(id: tokio::task::Id) -> u64 {
        // `tokio-dtrace` relies on the ability to cast a [`tokio::task::Id`] to a
        // [`u64`] in order to pass a task ID to DTrace probes as an integer. This
        // code checks that the sizes and alignments of the types are compatible. If
        // they are not, it is a build error, indicating that the casts are potentially
        // unsound. This may occur if Tokio has changed the representation of the
        // [`tokio::task::Id`] type, which is unlikely, but always possible.
        unsafe {
            // SAFETY:
            // * Size an alignment is guaranteed by asserts below, which
            //   leaves the question of value validity.
            // * Based on training and experience, I know that a
            //   `tokio::task::Id` is represented as a single `NonZeroU64`.
            // * However we use a u64 as that will be sound even if tokio
            //   starts producing the zero value.
            const {
                assert!(size_of::<tokio::task::Id>() == size_of::<u64>());
                assert!(align_of::<tokio::task::Id>() == align_of::<u64>());
            };
            std::mem::transmute::<_, u64>(id)
        }
    }
}

#[usdt::provider(provider = "tokio")]
#[allow(non_snake_case)]
mod probes {
    fn task__spawn(task_id: u64, file: String, line: u32, col: u32) {}
    fn task__poll__start(task_id: u64, file: String, line: u32, col: u32) {}
    fn task__poll__end(task_id: u64, file: String, line: u32, col: u32) {}
    fn task__terminate(task_id: u64, file: String, line: u32, col: u32) {}

    fn worker__thread__start() {}
    fn worker__thread__stop() {}
    fn worker__thread__park() {}
    fn worker__thread__unpark() {}
}
