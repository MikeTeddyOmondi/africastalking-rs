pub mod airtime;
pub mod application;
/// Module implementations for AfricasTalking services
pub mod sms;
pub mod data;
pub mod ussd;

// Re-export modules
pub use airtime::AirtimeModule;
pub use application::ApplicationModule;
pub use sms::SmsModule;
pub use data::DataModule;
pub use ussd::*;

// TODO: split modules into optional features

// Modules not implemented
// pub mod voice;
// pub mod payments;
// pub mod chat;
// pub mod insights;
