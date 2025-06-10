

/**
 * @module Support (LanguageFeatures/Handlers)
 * @description Contains the invocation logic for each specific language feature.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod ProvideHover;
// ... other invocation logic files

pub use self::ProvideHover::ProvideHoverLogic;
// ...
