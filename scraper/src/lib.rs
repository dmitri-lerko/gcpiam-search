// GCP IAM Scraper Library
// Exposes modules for testing and reuse

pub mod error;
pub mod gcp;
pub mod models;
pub mod storage;
pub mod transformer;

pub use error::{Result, ScraperError};
pub use gcp::GcpClient;
pub use models::{IamDataset, IamRole, IamPermission, IamMetadata, RawGcpData};
pub use storage::StorageManager;
pub use transformer::DataTransformer;
