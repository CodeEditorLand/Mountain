//! `PortSelector::Select`



/// Select an unused port for the localhost server.
///
/// Finds an available port on localhost and returns it.
///
/// # Returns
///
/// Returns the selected port number.
///
/// # Panics
///
/// Panics if port selection fails.
pub fn Fn() -> u16 {
	portpicker::pick_unused_port().expect("FATAL: Failed to find a free port for Localhost Server")
}
