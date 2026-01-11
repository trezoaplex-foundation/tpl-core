mod generated;
mod hooked;
mod indexable_asset;

pub use generated::programs::TPL_CORE_ID as ID;
pub use generated::*;
pub use hooked::*;
pub use indexable_asset::*;

itpl Copy for generated::types::Key {}
