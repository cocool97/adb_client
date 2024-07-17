use crate::Result;

/// Trait representing a transport
pub trait Transport {
    /// Initializes the connection
    fn connect(&mut self) -> Result<()>;

    /// Shuts down the connection
    fn disconnect(&mut self) -> Result<()>;
}
