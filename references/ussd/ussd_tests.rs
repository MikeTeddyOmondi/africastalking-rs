//! Comprehensive test suite for USSD module

#[cfg(test)]
mod ussd_tests {
    use africastalking::ussd::*;

    // ========================================================================
    // UssdRequest Tests
    // ========================================================================

    #[test]
    fn test_ussd_request_creation() {
        let request = UssdRequest::new(
            "session123",
            "*384*123#",
            "+254712345678",
            "1*2",
            "63902",
        );

        assert_eq!(request.session_id, "session123");
        assert_eq!(request.service_code, "*384*123#");
        assert_eq!(request.phone_number, "+254712345678");
        assert_eq!(request.text, "1*2");
        assert_eq!(request.network_code, "63902");
    }

    #[test]
    fn test_is_initial() {
        let initial = UssdRequest::new("s1", "*123#", "+254700000000", "", "63902");
        assert!(initial.is_initial());

        let not_initial = UssdRequest::new("s1", "*123#", "+254700000000", "1", "63902");
        assert!(!not_initial.is_initial());
    }

    #[test]
    fn test_depth() {
        let cases = vec![
            ("", 0),
            ("1", 1),
            ("1*2", 2),
            ("1*2*3", 3),
            ("1*2*3*4*5", 5),
        ];

        for (text, expected_depth) in cases {
            let request = UssdRequest::new("s1", "*123#", "+254700000000", text, "63902");
            assert_eq!(
                request.depth(),
                expected_depth,
                "Failed for text: '{}'",
                text
            );
        }
    }

    #[test]
    fn test_current_input() {
        let empty = UssdRequest::new("s1", "*123#", "+254700000000", "", "63902");
        assert_eq!(empty.current_input(), None);

        let single = UssdRequest::new("s1", "*123#", "+254700000000", "1", "63902");
        assert_eq!(single.current_input(), Some("1"));

        let multiple = UssdRequest::new("s1", "*123#", "+254700000000", "1*2*3", "63902");
        assert_eq!(multiple.current_input(), Some("3"));
    }

    #[test]
    fn test_navigation_path() {
        let request = UssdRequest::new("s1", "*123#", "+254700000000", "1*2*3*4", "63902");
        assert_eq!(request.navigation_path(), vec!["1", "2", "3", "4"]);

        let empty = UssdRequest::new("s1", "*123#", "+254700000000", "", "63902");
        assert_eq!(empty.navigation_path(), Vec::<&str>::new());
    }

    #[test]
    fn test_matches_path() {
        let request = UssdRequest::new("s1", "*123#", "+254700000000", "1*2*3", "63902");
        assert!(request.matches_path("1*2*3"));
        assert!(!request.matches_path("1*2"));
        assert!(!request.matches_path("1*2*3*4"));
    }

    #[test]
    fn test_starts_with_path() {
        let request = UssdRequest::new("s1", "*123#", "+254700000000", "1*2*3", "63902");
        assert!(request.starts_with_path("1"));
        assert!(request.starts_with_path("1*2"));
        assert!(request.starts_with_path("1*2*3"));
        assert!(!request.starts_with_path("2"));
        assert!(!request.starts_with_path("1*3"));
    }

    // ========================================================================
    // UssdResponse Tests
    // ========================================================================

    #[test]
    fn test_response_continues() {
        let response = UssdResponse::continues("Enter your PIN");
        assert!(response.is_continuing());
        assert!(!response.is_ending());
        assert_eq!(response.message(), "Enter your PIN");
        assert_eq!(response.to_string(), "CON Enter your PIN");
    }

    #[test]
    fn test_response_ends() {
        let response = UssdResponse::ends("Thank you!");
        assert!(!response.is_continuing());
        assert!(response.is_ending());
        assert_eq!(response.message(), "Thank you!");
        assert_eq!(response.to_string(), "END Thank you!");
    }

    #[test]
    fn test_response_with_multiline() {
        let message = "Welcome\nLine 2\nLine 3";
        let response = UssdResponse::continues(message);
        assert_eq!(response.to_string(), format!("CON {}", message));
    }

    // ========================================================================
    // UssdMenu Tests
    // ========================================================================

    #[test]
    fn test_menu_builder() {
        let menu = UssdMenu::new("Select an option")
            .add_option("1", "Account")
            .add_option("2", "Services")
            .build_continue();

        let expected = "CON Select an option\n1. Account\n2. Services";
        assert_eq!(menu.to_string(), expected);
        assert!(menu.is_continuing());
    }

    #[test]
    fn test_menu_builder_end() {
        let menu = UssdMenu::new("Final options")
            .add_option("1", "Yes")
            .add_option("2", "No")
            .build_end();

        let expected = "END Final options\n1. Yes\n2. No";
        assert_eq!(menu.to_string(), expected);
        assert!(menu.is_ending());
    }

    #[test]
    fn test_menu_multiple_options() {
        let options = vec![("1", "Option A"), ("2", "Option B"), ("3", "Option C")];

        let menu = UssdMenu::new("Choose:")
            .add_options(options)
            .build_continue();

        let expected = "CON Choose:\n1. Option A\n2. Option B\n3. Option C";
        assert_eq!(menu.to_string(), expected);
    }

    #[test]
    fn test_menu_empty_options() {
        let menu = UssdMenu::new("Just a message").build_continue();
        assert_eq!(menu.to_string(), "CON Just a message");
    }

    // ========================================================================
    // NetworkCode Tests
    // ========================================================================

    #[test]
    fn test_network_code_parsing() {
        let test_cases = vec![
            ("63902", NetworkCode::SafaricomKenya, "Safaricom Kenya", "Kenya"),
            ("62001", NetworkCode::MtnGhana, "MTN Ghana", "Ghana"),
            ("62130", NetworkCode::MtnNigeria, "MTN Nigeria", "Nigeria"),
            ("99999", NetworkCode::Athena, "Athena (Sandbox)", "Sandbox"),
        ];

        for (code, expected_variant, expected_name, expected_country) in test_cases {
            let network = NetworkCode::from_code(code);
            assert_eq!(network, expected_variant, "Failed for code: {}", code);
            assert_eq!(network.name(), expected_name);
            assert_eq!(network.country(), expected_country);
        }
    }

    #[test]
    fn test_network_code_unknown() {
        let unknown = NetworkCode::from_code("00000");
        assert!(matches!(unknown, NetworkCode::Unknown(_)));
        assert_eq!(unknown.name(), "Unknown Network");
        assert_eq!(unknown.country(), "Unknown");
    }

    #[test]
    fn test_network_code_display() {
        let network = NetworkCode::from_code("63902");
        assert_eq!(format!("{}", network), "Safaricom Kenya");
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[test]
    fn test_typical_flow_scenario() {
        // Simulate a typical USSD flow

        // Step 1: Initial request
        let req1 = UssdRequest::new("sess1", "*123#", "+254700000000", "", "63902");
        assert!(req1.is_initial());

        // Step 2: User selects option 1
        let req2 = UssdRequest::new("sess1", "*123#", "+254700000000", "1", "63902");
        assert_eq!(req2.depth(), 1);
        assert_eq!(req2.current_input(), Some("1"));

        // Step 3: User selects option 2 from submenu
        let req3 = UssdRequest::new("sess1", "*123#", "+254700000000", "1*2", "63902");
        assert_eq!(req3.depth(), 2);
        assert_eq!(req3.current_input(), Some("2"));
        assert!(req3.matches_path("1*2"));

        // Step 4: User enters data
        let req4 = UssdRequest::new("sess1", "*123#", "+254700000000", "1*2*500", "63902");
        assert_eq!(req4.depth(), 3);
        assert_eq!(req4.current_input(), Some("500"));
    }

    #[test]
    fn test_handler_pattern() {
        fn simple_handler(request: &UssdRequest) -> UssdResponse {
            match request.text.as_str() {
                "" => UssdMenu::new("Welcome")
                    .add_option("1", "Balance")
                    .add_option("2", "Exit")
                    .build_continue(),
                "1" => UssdResponse::ends("Your balance: KES 1000"),
                "2" => UssdResponse::ends("Goodbye!"),
                _ => UssdResponse::ends("Invalid option"),
            }
        }

        // Test initial
        let req = UssdRequest::new("s1", "*123#", "+254700000000", "", "63902");
        let resp = simple_handler(&req);
        assert!(resp.is_continuing());
        assert!(resp.to_string().contains("Welcome"));

        // Test balance
        let req = UssdRequest::new("s1", "*123#", "+254700000000", "1", "63902");
        let resp = simple_handler(&req);
        assert!(resp.is_ending());
        assert!(resp.to_string().contains("1000"));

        // Test exit
        let req = UssdRequest::new("s1", "*123#", "+254700000000", "2", "63902");
        let resp = simple_handler(&req);
        assert!(resp.is_ending());
        assert!(resp.to_string().contains("Goodbye"));
    }

    // ========================================================================
    // Serialization Tests
    // ========================================================================

    #[test]
    fn test_request_deserialization() {
        let json = r#"{
            "sessionId": "ATUid_12345",
            "serviceCode": "*384*123#",
            "phoneNumber": "+254712345678",
            "text": "1*2",
            "networkCode": "63902"
        }"#;

        let request: UssdRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.session_id, "ATUid_12345");
        assert_eq!(request.service_code, "*384*123#");
        assert_eq!(request.phone_number, "+254712345678");
        assert_eq!(request.text, "1*2");
        assert_eq!(request.network_code, "63902");
    }

    #[test]
    fn test_notification_deserialization() {
        let json = r#"{
            "date": "2024-12-10 10:30:00",
            "sessionId": "ATUid_12345",
            "serviceCode": "*384*123#",
            "networkCode": "63902",
            "phoneNumber": "+254712345678",
            "status": "Success",
            "cost": "KES 0.50",
            "durationInMillis": "15000",
            "hopsCount": 3,
            "input": "1*2",
            "lastAppResponse": "END Your balance is KES 1000"
        }"#;

        let notification: UssdNotification = serde_json::from_str(json).unwrap();
        assert_eq!(notification.session_id, "ATUid_12345");
        assert_eq!(notification.status, UssdSessionStatus::Success);
        assert_eq!(notification.hops_count, 3);
        assert_eq!(notification.error_message, None);
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_empty_text_variations() {
        let request = UssdRequest::new("s1", "*123#", "+254700000000", "", "63902");
        assert!(request.is_initial());
        assert_eq!(request.depth(), 0);
        assert_eq!(request.current_input(), None);
        assert_eq!(request.navigation_path().len(), 0);
    }

    #[test]
    fn test_special_characters_in_response() {
        let response = UssdResponse::continues("Amount: KES 1,234.56\nFee: KES 10.00");
        assert!(response.to_string().contains("1,234.56"));
    }

    #[test]
    fn test_long_menu() {
        let menu = UssdMenu::new("Select:")
            .add_option("1", "Option 1")
            .add_option("2", "Option 2")
            .add_option("3", "Option 3")
            .add_option("4", "Option 4")
            .add_option("5", "Option 5")
            .add_option("6", "Option 6")
            .build_continue();

        let result = menu.to_string();
        assert!(result.contains("1. Option 1"));
        assert!(result.contains("6. Option 6"));
    }

    #[test]
    fn test_unicode_in_responses() {
        let response = UssdResponse::ends("Asante! 🎉 Thank you!");
        // Note: Special characters may not render on all networks
        assert!(response.to_string().contains("Asante"));
    }
}

// ========================================================================
// Benchmark Tests (optional, requires criterion)
// ========================================================================

#[cfg(all(test, feature = "bench"))]
mod bench {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_request_parsing(c: &mut Criterion) {
        c.bench_function("parse navigation path", |b| {
            let request = UssdRequest::new("s1", "*123#", "+254700000000", "1*2*3*4*5", "63902");
            b.iter(|| {
                black_box(request.navigation_path());
            });
        });
    }

    criterion_group!(benches, bench_request_parsing);
    criterion_main!(benches);
}
