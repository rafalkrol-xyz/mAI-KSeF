//! Loading an [`Invoice`] from a TOML config file.
//!
//! The intended workflow: keep `template.example.toml` in git with fake rates,
//! copy it to `invoice.toml` (gitignored), and edit `invoice.toml` each month.

use std::path::Path;

use crate::model::Invoice;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("could not read invoice config at {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse invoice config: {0}")]
    Parse(#[from] toml::de::Error),
}

/// Load and parse an invoice from a TOML file.
pub fn load_invoice(path: impl AsRef<Path>) -> Result<Invoice, ConfigError> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.display().to_string(),
        source,
    })?;
    let invoice: Invoice = toml::from_str(&text)?;
    Ok(invoice)
}
