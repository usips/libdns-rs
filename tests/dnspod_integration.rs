//! Integration tests for DNSPod provider.
//!
//! These tests require valid DNSPod credentials and are ignored by default.
//! To run them:
//!
//! 1. Create a `.env` file in the project root with:
//!    ```
//!    DNSPOD_SECRET_ID=your_secret_id
//!    DNSPOD_SECRET_KEY=your_secret_key
//!    DNSPOD_TEST_PROGRAM=YourAppName
//!    DNSPOD_TEST_VERSION=1.0.0
//!    DNSPOD_TEST_EMAIL=your@email.com
//!    ```
//!
//! 2. Run with: `cargo test --features dnspod -- --ignored`

#![cfg(feature = "dnspod")]

use libdns::dnspod::{ClientConfig, DnspodProvider};
use libdns::{Provider, Zone};
use std::env;

/// Helper to load credentials from environment.
/// Returns None if credentials are not available.
fn get_test_provider() -> Option<DnspodProvider> {
    // Load .env file if present (ignore errors if file doesn't exist)
    let _ = dotenvy::dotenv();

    let secret_id = env::var("DNSPOD_SECRET_ID").ok()?;
    let secret_key = env::var("DNSPOD_SECRET_KEY").ok()?;
    let program = env::var("DNSPOD_TEST_PROGRAM").unwrap_or_else(|_| "libdns-test".to_string());
    let version = env::var("DNSPOD_TEST_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
    let email = env::var("DNSPOD_TEST_EMAIL").unwrap_or_else(|_| "test@example.com".to_string());

    let login_token = format!("{},{}", secret_id, secret_key);
    let config = ClientConfig::new(program, version, email);

    DnspodProvider::new(&login_token, &config).ok()
}

/// Test that we can authenticate and list zones.
/// This test is ignored by default - run with `cargo test -- --ignored`
#[tokio::test]
#[ignore = "requires DNSPOD credentials in .env"]
async fn test_list_zones() {
    let provider = get_test_provider()
        .expect("DNSPOD credentials not found. Set DNSPOD_SECRET_ID and DNSPOD_SECRET_KEY in .env");

    let result = provider.list_zones().await;

    match result {
        Ok(zones) => {
            println!("Found {} zones", zones.len());
            for zone in &zones {
                println!("  - {} (ID: {})", zone.domain(), zone.id());
            }
        }
        Err(e) => {
            panic!("Failed to list zones: {:?}", e);
        }
    }
}

/// Test that authentication failure is handled properly.
#[tokio::test]
async fn test_invalid_credentials() {
    let config = ClientConfig::new("libdns-test", "0.1.0", "test@example.com");
    let provider = DnspodProvider::new("invalid_id,invalid_key", &config)
        .expect("Client creation should succeed");

    let result = provider.list_zones().await;

    // Should fail with unauthorized or similar error
    assert!(result.is_err(), "Expected error with invalid credentials");
}

/// Test ClientConfig user agent format.
#[test]
fn test_client_config_user_agent() {
    let config = ClientConfig::new("MyApp", "1.2.3", "dev@example.com");
    assert_eq!(config.user_agent(), "MyApp/1.2.3 (dev@example.com)");
}
