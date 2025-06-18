// @module RunTime
// @description This module defines the concrete `ApplicationRunTime` for the
// Mountain application.
//
// The `ApplicationRunTime` is the core execution engine that is powered by the
// `Echo` scheduler. It implements the `ApplicationRunTimeTrait` from the
// `Common` crate and is responsible for running all `ActionEffect`s.
//

#![allow(non_snake_case, non_camel_case_types)]

// The concrete, Echo-based RunTime for the Mountain application.
pub mod ApplicationRunTime;
