pub use self::data::InformationData;
mod data;

#[cfg(feature = "setup")]
pub use self::setup::INFO;
#[cfg(feature = "setup")]
pub use self::setup::info_data;
#[cfg(feature = "setup")]
pub mod setup;

#[cfg(feature = "vmm")]
pub use self::vmm::info_data;
#[cfg(feature = "vmm")]
pub mod vmm;
