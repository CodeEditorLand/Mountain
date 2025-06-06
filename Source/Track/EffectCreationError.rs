
// Defines the error types that can occur during the creation of an ActionEffect
// from an incoming RPC or command.

#![allow(non_snake_case, non_camel_case_types)]

pub enum EffectCreationError {
	// Indicates that there is no mapping from the incoming method/command name
	// to a corresponding ActionEffect. This signals the dispatcher to try
	// another handling mechanism (like a direct RPC handler).
	NoEffectMapping,

	// Indicates an error occurred while parsing the parameters required to
	// construct the ActionEffect. The contained string provides details.
	ParameterParseError(String),
}
