

/**
 * @module runtime
 * @description This module defines the concrete `AppRuntime` for the Mountain application.
 *
 * The `AppRuntime` is the core execution engine that is powered by the `Echo`
 * scheduler. It implements the `AppRuntimeTrait` from the `Common` crate and is
 * responsible for running all `ActionEffect`s.
 */

#![allow(non_snake_case, non_camel_case_types)]

// The concrete, Echo-based runtime for the Mountain application.
pub mod AppRuntime;
