//! SQLite storage implementation for settings.

mod model;
mod repository;

pub use model::AppSettingDB;
pub use repository::SettingsRepository;

// Re-export trait from core for convenience
pub use sensible_folio_core::settings::SettingsRepositoryTrait;
