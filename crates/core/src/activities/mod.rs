//! Activities module - domain models, services, and traits.

mod activities_constants;
mod activities_errors;
mod activities_model;
mod activities_service;
mod activities_traits;
mod bank_csv_parser;
mod bank_mapper;
mod compiler;
mod csv_parser;
mod idempotency;
mod import_run_model;
mod ofx_parser;
mod qif_parser;

#[cfg(test)]
mod activities_service_tests;

#[cfg(test)]
mod activities_model_tests;

pub use activities_constants::*;
pub use activities_errors::ActivityError;
pub use activities_model::{
    parse_decimal_string_tolerant, Activity, ActivityBulkIdentifierMapping,
    ActivityBulkMutationError, ActivityBulkMutationRequest, ActivityBulkMutationResult,
    ActivityDetails, ActivityImport, ActivitySearchResponse, ActivitySearchResponseMeta,
    ActivityStatus, ActivityType, ActivityUpdate, ActivityUpsert, BulkUpsertResult,
    ImportActivitiesResult, ImportActivitiesSummary, ImportMapping, ImportMappingData, IncomeData,
    NewActivity, PrepareActivitiesResult, Sort, SymbolInput,
};
pub use activities_service::ActivityService;
pub use activities_traits::{ActivityRepositoryTrait, ActivityServiceTrait};
pub use bank_csv_parser::{parse_australian_bank_csv, BankTransaction, ParsedBankCsvResult};
pub use bank_mapper::{
    map_bank_transactions, BankMapperConfig, BankTransactionInput, MappedActivity,
};
pub use compiler::{ActivityCompiler, DefaultActivityCompiler};
pub use csv_parser::{parse_csv, ParseConfig, ParseError, ParsedCsvResult};
pub use idempotency::{
    compute_activity_idempotency_key, compute_idempotency_key, generate_manual_idempotency_key,
};
pub use import_run_model::{
    ImportRun, ImportRunMode, ImportRunRepositoryTrait, ImportRunStatus, ImportRunSummary,
    ImportRunType, ReviewMode,
};
pub use ofx_parser::{parse_ofx, OfxParseResult, OfxTransaction};
pub use qif_parser::{parse_qif, QifParseResult, QifTransaction};
