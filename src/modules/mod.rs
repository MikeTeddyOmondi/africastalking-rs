pub mod airtime;
pub mod application;
/// Module implementations for AfricasTalking services
pub mod sms;
pub mod data;
pub mod ussd;
pub mod voice;

// Re-export modules
pub use airtime::AirtimeModule;
pub use application::ApplicationModule;
pub use sms::SmsModule;
pub use data::DataModule;
pub use ussd::*;
pub use voice::*;

// TODO: split modules into optional features

// Modules not implemented
// pub mod payments;
// pub mod chat;
// pub mod insights;
