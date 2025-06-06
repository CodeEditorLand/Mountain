// File: EsmInterceptor/mod.rs
// Declares and exports modules related to the ES Module (ESM) interception
// mechanism. This system allows Cocoon to intercept `import "vscode"`
// statements in extensions and provide the correct, shimmed API instance.

#![allow(non_snake_case, non_camel_case_types)]

// This module contains the main interceptor class.
mod EsmInterceptor;
// This module contains the dynamic script that gets served for `import
// "vscode"`.
mod Dynamic;

// Re-export the primary interceptor class.
pub use self::EsmInterceptor::CocoonNodeModuleESMInterceptor;
