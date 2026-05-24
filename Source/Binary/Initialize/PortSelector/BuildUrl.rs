//! `PortSelector::BuildUrl`



/// Build localhost URL from port number.
///
/// Creates a localhost URL from the provided port number.
///
/// # Arguments
///
/// * `Port` - Port number
///
/// # Returns
///
/// Returns the localhost URL string.
pub fn Fn(Port:u16) -> String { format!("http://localhost:{}", Port) }
