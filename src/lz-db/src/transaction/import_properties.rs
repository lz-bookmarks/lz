//! Properties imported from other systems.
//!
//! These are kept around verbatim, ensuring that they can be
//! re-translated into our internal format in a higher-fidelity way as
//! we evolve the system.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The systems we can import from.
#[derive(Serialize, Deserialize, PartialEq, Hash, Eq, Debug, Clone)]
pub enum ImportableSystem {
    Linkding,
}

/// Properties imported from other systems.
///
/// If the bookmark originated in another bookmark-keeping system,
/// this structure keeps their original values around for future
/// higher-fidelity importing purpose.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ImportProperties {
    pub by_system: HashMap<ImportableSystem, HashMap<String, serde_json::Value>>,
}
