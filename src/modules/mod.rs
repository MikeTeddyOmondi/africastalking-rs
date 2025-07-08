pub mod airtime;
pub mod application;
/// Module implementations for AfricasTalking services
pub mod sms;

// Re-export modules
pub use airtime::AirtimeModule;
pub use application::ApplicationModule;
pub use sms::SmsModule;

// TODO: split modules into optional features

// Modules not implemented
// pub mod voice;
// pub mod payments;
// pub mod data;
// pub mod chat;
// pub mod insights;
// pub mod ussd;
