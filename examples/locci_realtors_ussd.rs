//! Example: Locci Realtors USSD application
//!
//! A complete USSD interface for the Locci Realtors property management platform.
//! Tenants can browse available properties and check their active leases without
//! needing internet access — only a basic phone and a USSD session.
//!
//! # Flow overview
//!
//! ```text
//! Main Menu
//! ├── 1. Browse Properties
//! │   ├── 1. By Property Type  → Apartment / House / Townhouse / Studio / Commercial
//! │   ├── 2. By Rent Range     → <10k / 10-30k / 30-60k / 60k+
//! │   ├── 3. By City           → Nairobi / Mombasa / Kisumu / Nakuru / Other
//! │   └── 4. Show All          → first 3 listings
//! ├── 2. My Leases             → active leases looked up by caller phone number
//! └── 3. Help                  → contact information
//! ```

use africastalking::ussd::{UssdMenu, UssdNotification, UssdRequest, UssdResponse};
use axum::{
    Form, Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
};
use mongodb::{
    Client,
    bson::{Document, doc, oid::ObjectId},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;

// ---------------------------------------------------------------------------
// Domain structs — mirroring the Mongoose schemas in models.ts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PropertyLocation {
    pub address: String,
    pub city: String,
    pub county: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

/// Mirrors the `propertyType` enum in the Mongoose PropertySchema.
/// Stored in MongoDB as lowercase strings: "apartment", "house", etc.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PropertyType {
    Apartment,
    House,
    Townhouse,
    Studio,
    Commercial,
}

impl PropertyType {
    /// Returns the lowercase MongoDB string value for use in query filters.
    fn as_str(&self) -> &'static str {
        match self {
            PropertyType::Apartment => "apartment",
            PropertyType::House => "house",
            PropertyType::Townhouse => "townhouse",
            PropertyType::Studio => "studio",
            PropertyType::Commercial => "commercial",
        }
    }

    /// Returns a human-readable display label.
    fn label(&self) -> &'static str {
        match self {
            PropertyType::Apartment => "Apartment",
            PropertyType::House => "House",
            PropertyType::Townhouse => "Townhouse",
            PropertyType::Studio => "Studio",
            PropertyType::Commercial => "Commercial",
        }
    }
}

/// Mirrors the `Property` Mongoose model.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub description: String,
    pub property_type: PropertyType,
    pub location: PropertyLocation,
    pub monthly_rent: f64,
    pub security_deposit: f64,
    pub bedrooms: u32,
    pub bathrooms: u32,
    pub square_meters: Option<f64>,
    pub amenities: Vec<String>,
    pub images: Vec<String>,
    pub available: bool,
    pub landlord_id: String,
    pub contact_number: Option<String>,
}

/// Mirrors the `Tenant` Mongoose model.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tenant {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub id_number: String,
}

/// Mirrors the `status` enum in the Mongoose LeaseSchema.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LeaseStatus {
    Pending,
    Active,
    Expired,
    Terminated,
}

impl LeaseStatus {
    fn label(&self) -> &'static str {
        match self {
            LeaseStatus::Pending => "Pending",
            LeaseStatus::Active => "Active",
            LeaseStatus::Expired => "Expired",
            LeaseStatus::Terminated => "Terminated",
        }
    }
}

/// Mirrors the `Lease` Mongoose model.
///
/// `property_id` and `tenant_id` are stored as `String` (hex) here because
/// Mongoose may have serialised the ObjectId references to plain strings before
/// insertion, so using `ObjectId` would silently fail deserialization and drop
/// every document from the cursor.  We convert to `ObjectId` only when we need
/// to query the `_id` field on another collection.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Lease {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub property_id: String,
    pub tenant_id: String,
    pub start_date: String,
    pub end_date: String,
    pub monthly_rent: f64,
    pub security_deposit_paid: bool,
    pub status: LeaseStatus,
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    db: mongodb::Database,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let mongo_uri = std::env::var("MONGODB_URI")
        .expect("MONGODB_URI must be set in environment or .env file");
    let db_name = std::env::var("DB_NAME")
        .unwrap_or_else(|_| "locci_realtors".to_string());

    let client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");
    let db = client.database(&db_name);

    let state = Arc::new(AppState { db });

    let app = Router::new()
        .route("/api/v1/ussd", post(handle_ussd))
        .route("/api/v1/ussd/notify", post(handle_ussd_notification))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:4949").await.unwrap();
    println!("Locci Realtors USSD server running on http://localhost:4949");

    axum::serve(listener, app).await.unwrap();
}

// ---------------------------------------------------------------------------
// Axum handlers
// ---------------------------------------------------------------------------

async fn handle_ussd(
    State(state): State<Arc<AppState>>,
    Form(request): Form<UssdRequest>,
) -> Response {
    println!("USSD Request: {:?}", request);
    let response = route_ussd_request(&state, &request).await;
    (
        [(axum::http::header::CONTENT_TYPE, "text/plain")],
        response.to_string(),
    )
        .into_response()
}

async fn handle_ussd_notification(
    State(_state): State<Arc<AppState>>,
    Form(notification): Form<UssdNotification>,
) -> Response {
    println!("USSD Notification: {:?}", notification);
    match notification.status {
        africastalking::ussd::UssdSessionStatus::Success => {
            println!(
                "Session OK  | phone={} duration={}ms hops={}",
                notification.phone_number,
                notification.duration_in_millis,
                notification.hops_count,
            );
        }
        africastalking::ussd::UssdSessionStatus::Incomplete => {
            println!("Session incomplete | phone={}", notification.phone_number);
        }
        africastalking::ussd::UssdSessionStatus::Failed => {
            println!(
                "Session failed | phone={} err={:?}",
                notification.phone_number, notification.error_message
            );
        }
    }
    axum::http::StatusCode::OK.into_response()
}

// ---------------------------------------------------------------------------
// USSD router
// ---------------------------------------------------------------------------

/// Routes USSD requests based on the accumulated navigation path in `request.text`.
///
/// Africa's Talking concatenates every user input with `*`, so navigating
/// Main → Properties → By Type → Apartment produces `text = "1*1*1"`.
async fn route_ussd_request(state: &AppState, request: &UssdRequest) -> UssdResponse {
    if request.is_initial() {
        return main_menu();
    }

    match request.text.as_str() {
        // ── 1. Browse Properties ───────────────────────────────────────────
        "1" => UssdMenu::new("Browse Properties")
            .add_option("1", "By Property Type")
            .add_option("2", "By Rent Range")
            .add_option("3", "By City")
            .add_option("4", "Show All")
            .build_continue(),

        // 1*1 — By Type sub-menu
        "1*1" => UssdMenu::new("Select Property Type")
            .add_option("1", "Apartment")
            .add_option("2", "House")
            .add_option("3", "Townhouse")
            .add_option("4", "Studio")
            .add_option("5", "Commercial")
            .build_continue(),

        "1*1*1" => list_by_type(state, PropertyType::Apartment).await,
        "1*1*2" => list_by_type(state, PropertyType::House).await,
        "1*1*3" => list_by_type(state, PropertyType::Townhouse).await,
        "1*1*4" => list_by_type(state, PropertyType::Studio).await,
        "1*1*5" => list_by_type(state, PropertyType::Commercial).await,

        // 1*2 — By Rent Range sub-menu
        "1*2" => UssdMenu::new("Select Rent Range (KES/mo)")
            .add_option("1", "Under 10,000")
            .add_option("2", "10,000 - 30,000")
            .add_option("3", "30,000 - 60,000")
            .add_option("4", "Above 60,000")
            .build_continue(),

        "1*2*1" => list_by_rent(state, 0.0, 10_000.0).await,
        "1*2*2" => list_by_rent(state, 10_000.0, 30_000.0).await,
        "1*2*3" => list_by_rent(state, 30_000.0, 60_000.0).await,
        "1*2*4" => list_by_rent(state, 60_000.0, f64::MAX).await,

        // 1*3 — By City sub-menu
        "1*3" => UssdMenu::new("Select City")
            .add_option("1", "Nairobi")
            .add_option("2", "Mombasa")
            .add_option("3", "Kisumu")
            .add_option("4", "Nakuru")
            .add_option("5", "Other Cities")
            .build_continue(),

        "1*3*1" => list_by_city(state, "Nairobi").await,
        "1*3*2" => list_by_city(state, "Mombasa").await,
        "1*3*3" => list_by_city(state, "Kisumu").await,
        "1*3*4" => list_by_city(state, "Nakuru").await,
        "1*3*5" => list_by_city(state, "").await, // all cities not in the list above

        // 1*4 — Show All (first 3)
        "1*4" => list_all(state).await,

        // ── 2. My Leases ───────────────────────────────────────────────────
        "2" => {
            println!("Looking up leases for phone number: {}", request.phone_number);
            my_leases(state, &request.phone_number).await
        }

        // ── 3. Help ────────────────────────────────────────────────────────
        "3" => UssdResponse::ends(
            "Locci Realtors Support\nCall: 0800-123-456\nWeb: realtors.locci.cloud\nEmail: help@locci.cloud",
        ),

        _ => UssdResponse::ends("Invalid option. Dial again to restart."),
    }
}

fn main_menu() -> UssdResponse {
    UssdMenu::new("Welcome to Locci Realtors")
        .add_option("1", "Browse Properties")
        .add_option("2", "My Leases")
        .add_option("3", "Help")
        .build_continue()
}

// ---------------------------------------------------------------------------
// Property listing helpers
// ---------------------------------------------------------------------------

/// Formats up to 3 properties into a compact USSD-friendly string.
/// USSD pages are typically ≤182 characters, so we keep entries terse.
fn format_properties(properties: &[Property]) -> String {
    if properties.is_empty() {
        return "No properties found.".to_string();
    }

    let mut lines: Vec<String> = properties.iter().take(3).map(|p| {
        let contact = p
            .contact_number
            .as_deref()
            .map(|n| format!(" Tel:{}", n))
            .unwrap_or_default();
        format!(
            "{} | {}bd | KES {:.0}/mo | {}{}",
            p.title,
            p.bedrooms,
            p.monthly_rent,
            p.location.city,
            contact,
        )
    }).collect();

    if properties.len() > 3 {
        lines.push(format!(
            "+{} more: realtors.locci.cloud",
            properties.len() - 3
        ));
    }

    lines.join("\n")
}

async fn list_by_type(state: &AppState, property_type: PropertyType) -> UssdResponse {
    let filter = doc! {
        "available": true,
        "propertyType": property_type.as_str(),
    };
    match query_properties(state, filter).await {
        Ok(props) => {
            let header = format!("{}s Available\n", property_type.label());
            UssdResponse::ends(format!("{}{}", header, format_properties(&props)))
        }
        Err(e) => {
            eprintln!("list_by_type error: {e}");
            UssdResponse::ends("Could not load properties. Try again later.")
        }
    }
}

async fn list_by_rent(state: &AppState, min: f64, max: f64) -> UssdResponse {
    let filter = if max == f64::MAX {
        doc! { "available": true, "monthlyRent": { "$gte": min } }
    } else {
        doc! { "available": true, "monthlyRent": { "$gte": min, "$lte": max } }
    };
    let range_label = if max == f64::MAX {
        format!("KES {:.0}+/mo", min)
    } else {
        format!("KES {:.0}-{:.0}/mo", min, max)
    };
    match query_properties(state, filter).await {
        Ok(props) => UssdResponse::ends(format!(
            "Properties {}\n{}",
            range_label,
            format_properties(&props)
        )),
        Err(e) => {
            eprintln!("list_by_rent error: {e}");
            UssdResponse::ends("Could not load properties. Try again later.")
        }
    }
}

async fn list_by_city(state: &AppState, city: &str) -> UssdResponse {
    let filter = if city.is_empty() {
        // "Other Cities" — exclude the four named cities
        doc! {
            "available": true,
            "location.city": {
                "$nin": ["Nairobi", "Mombasa", "Kisumu", "Nakuru"]
            }
        }
    } else {
        doc! {
            "available": true,
            "location.city": { "$regex": city, "$options": "i" }
        }
    };
    let header = if city.is_empty() {
        "Other Cities".to_string()
    } else {
        format!("Properties in {}", city)
    };
    match query_properties(state, filter).await {
        Ok(props) => UssdResponse::ends(format!("{}\n{}", header, format_properties(&props))),
        Err(e) => {
            eprintln!("list_by_city error: {e}");
            UssdResponse::ends("Could not load properties. Try again later.")
        }
    }
}

async fn list_all(state: &AppState) -> UssdResponse {
    let filter = doc! { "available": true };
    match query_properties(state, filter).await {
        Ok(props) => UssdResponse::ends(format!(
            "Available Properties\n{}",
            format_properties(&props)
        )),
        Err(e) => {
            eprintln!("list_all error: {e}");
            UssdResponse::ends("Could not load properties. Try again later.")
        }
    }
}

// ---------------------------------------------------------------------------
// Lease lookup helper
// ---------------------------------------------------------------------------

/// Looks up the caller's active leases by matching their phone number against
/// the Tenant collection, then follows the ObjectId relationship to Lease.
///
/// All phone numbers in the system are stored in E.164 format (`+254XXXXXXXXX`),
/// which is also the format Africa's Talking delivers in `request.phone_number`,
/// so a direct equality match is sufficient — no format normalisation required.
/// Once the Tenant `_id` (ObjectId) is retrieved it is used as the `tenantId`
/// foreign key on the Lease documents.
async fn my_leases(state: &AppState, phone_number: &str) -> UssdResponse {
    println!("Looking up tenant for {}", phone_number);

    let tenants: mongodb::Collection<Tenant> = state.db.collection("tenants");

    let tenant = match tenants
        .find_one(doc! { "phone": phone_number })
        .await
    {
        Ok(Some(t)) => t,
        Ok(None) => {
            return UssdResponse::ends(
                "No account found for this number.\nVisit realtors.locci.cloud to register.",
            )
        }
        Err(e) => {
            eprintln!("my_leases tenant lookup error: {e}");
            return UssdResponse::ends("Could not fetch account. Try again later.");
        }
    };

    // The Tenant._id (ObjectId) is the foreign key stored in Lease.tenantId.
    // This is entirely separate from the phone number — we resolved the phone →
    // ObjectId bridge above; from here the relationship is purely by ObjectId.
    let tenant_oid = match tenant.id {
        Some(id) => id,
        None => {
            eprintln!("my_leases: tenant record has no _id — data integrity issue");
            return UssdResponse::ends("Account error. Please contact support.");
        }
    };

    println!("Found tenant '{}' (oid={})", tenant.name, tenant_oid);

    // Lease.tenantId may be stored as a BSON ObjectId (Mongoose default) or as
    // a plain hex string (if inserted via a JSON-serialised payload that already
    // converted the ObjectId to string).  Query both so the lookup is robust to
    // whichever format is present in the collection.
    let tenant_oid_str = tenant_oid.to_string();
    let leases: mongodb::Collection<Lease> = state.db.collection("leases");
    let mut cursor = match leases
        .find(doc! {
            "$and": [
                { "tenantId": { "$in": [tenant_oid, tenant_oid_str.as_str()] } },
                { "status": "active" }
            ]
        })
        .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("my_leases lease query error: {e}");
            return UssdResponse::ends("Could not fetch leases. Try again later.");
        }
    };

    let mut active_leases: Vec<Lease> = Vec::new();
    while let Ok(true) = cursor.advance().await {
        if let Ok(lease) = cursor.deserialize_current() {
            active_leases.push(lease);
        }
    }

    if active_leases.is_empty() {
        return UssdResponse::ends(format!(
            "Hi {}!\nNo active leases found.\nVisit realtors.locci.cloud",
            tenant.name
        ));
    }

    // Resolve property titles for each lease (up to 3)
    let properties: mongodb::Collection<Property> = state.db.collection("properties");
    let mut lines = vec![format!("Hi {}! Active Leases:", tenant.name)];

    for lease in active_leases.iter().take(3) {
        // property_id is a hex string; parse it back to ObjectId for the _id lookup.
        let prop_title = match ObjectId::parse_str(&lease.property_id) {
            Ok(oid) => properties
                .find_one(doc! { "_id": oid })
                .await
                .ok()
                .flatten()
                .map(|p| p.title)
                .unwrap_or_else(|| "Unknown Property".to_string()),
            Err(_) => "Unknown Property".to_string(),
        };

        lines.push(format!(
            "{} | KES {:.0}/mo | {}",
            prop_title,
            lease.monthly_rent,
            lease.status.label(),
        ));
    }

    if active_leases.len() > 3 {
        lines.push(format!(
            "+{} more: realtors.locci.cloud",
            active_leases.len() - 3
        ));
    }

    UssdResponse::ends(lines.join("\n"))
}

// ---------------------------------------------------------------------------
// Shared DB query
// ---------------------------------------------------------------------------

/// Queries the `properties` collection with the given filter, returning up to
/// 10 results sorted by most recently created first.
async fn query_properties(
    state: &AppState,
    filter: Document,
) -> Result<Vec<Property>, mongodb::error::Error> {
    let col: mongodb::Collection<Property> = state.db.collection("properties");
    let mut cursor = col
        .find(filter)
        .sort(doc! { "createdAt": -1 })
        .limit(10)
        .await?;

    let mut results = Vec::new();
    while let Ok(true) = cursor.advance().await {
        if let Ok(p) = cursor.deserialize_current() {
            results.push(p);
        }
    }
    Ok(results)
}
