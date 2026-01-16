use crate::{
    invoice::{Address, Buyer, Invoice, Product, Seller},
    pdf::generate_invoice_pdf,
};
use hmac::{Hmac, Mac};
use locale_rs::{Locale, datetime_formats::DateTime};
use sha2::Sha256;
use std::{
    fs,
    io::{self, Write},
};
use tempfile::NamedTempFile;

fn compute_hmac(key: &[u8], data: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).unwrap();
    mac.update(data);
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

fn make_test_invoice() -> Invoice {
    let seller_addr = Address {
        street: "Seller Street".to_string(),
        house_number: 67,
        code: 42069,
        town: "Rizzton".to_string(),
    };

    let buyer_addr = Address {
        street: "Buyer Street".to_string(),
        house_number: 67,
        code: 69420,
        town: "Rizzton".to_string(),
    };

    Invoice {
        number: "TEST-001".to_string(),
        date: DateTime {
            year: 2025,
            month: 7,
            day: 15,
            hour: 0,
            minute: 0,
            second: 0,
        },
        seller: Seller {
            name: "Test Seller".to_string(),
            address: seller_addr,
            vat_id: "VAT-TEST-123".to_string(),
            website: "test.example.com".to_string(),
        },
        buyer: Buyer {
            name: "Test Buyer".to_string(),
            address: buyer_addr,
            email: "buyer@example.com".to_string(),
        },
        payment_due: DateTime {
            year: 2025,
            month: 7,
            day: 15,
            hour: 0,
            minute: 0,
            second: 0,
        },
        delivery_date: DateTime {
            year: 2025,
            month: 7,
            day: 15,
            hour: 0,
            minute: 0,
            second: 0,
        },
        delivery_type: None,
        extra_info: vec![],
        payment_type: None,
        payment_info: vec![],
        products: vec![
            Product {
                description: "Widget".to_string(),
                units: 2,
                cost_per_unit: 10.0,
                tax_rate: 0.19,
                tax_exempt_reason: None,
            },
            Product {
                description: "Gadget".to_string(),
                units: 1,
                cost_per_unit: 20.0,
                tax_rate: 0.19,
                tax_exempt_reason: None,
            },
        ],
        locale: Locale::de,
    }
}

#[test]
fn test_generate_invoice_pdf_in_memory() {
    let invoice = make_test_invoice();
    let pdf_bytes = generate_invoice_pdf(
        &invoice,
        std::path::Path::new("./fonts/OpenSans-Medium.ttf"),
        None,
    )
    .unwrap();
    assert!(!pdf_bytes.is_empty(), "PDF bytes should not be empty");
}

#[test]
fn test_generate_and_save_pdf_tempfile() -> io::Result<()> {
    let invoice = make_test_invoice();

    let pdf_bytes = generate_invoice_pdf(
        &invoice,
        std::path::Path::new("./fonts/OpenSans-Medium.ttf"),
        None,
    )
    .unwrap();

    let mut tmp_file = NamedTempFile::new()?;
    tmp_file.write_all(&pdf_bytes)?;

    let metadata = tmp_file.as_file().metadata()?;
    assert!(metadata.len() > 0, "Temporary PDF file is empty");

    Ok(())
}

#[test]
fn validation_test() {
    let invoice_id = "100";
    let mut invoice = make_test_invoice();
    invoice.number = invoice_id.to_string();

    let pdf_bytes = generate_invoice_pdf(
        &invoice,
        std::path::Path::new("./fonts/OpenSans-Medium.ttf"),
        None,
    )
    .unwrap();

    let hash1 = compute_hmac(b"very secret secret", &pdf_bytes);
    let hash2 = compute_hmac(b"very secret secret", &pdf_bytes);

    assert_eq!(hash1, hash2, "HMAC should be stable across identical input");
}

#[test]
fn detect_breaking_changes() {
    if let Ok(pdf_bytes) = fs::read("old.pdf") {
        let hash = compute_hmac(b"secret", &pdf_bytes);
        assert_eq!(
            "fc87ddd5ea2064137bf17c65ef03e85e708f49a6778dd04f8a36a53a34a6b901",
            hash
        );
    } else {
        eprintln!("old.pdf not found, skipping breaking change test");
    }
}

#[test]
fn test_xml_against_itb_api() {
    let seller_addr = Address {
        street: "Main Street".to_string(),
        house_number: 123,
        code: 90210,
        town: "Anytown".to_string(),
    };

    let buyer_addr = Address {
        street: "Oak Avenue".to_string(),
        house_number: 456,
        code: 10001,
        town: "Cityville".to_string(),
    };

    let invoice = Invoice {
        number: "INV-2025-EXAMPLE".to_string(),
        date: DateTime { year: 2025, month: 7, day: 15, hour: 0, minute: 0, second: 0 },
        seller: Seller {
            name: "Example Corp".to_string(),
            address: seller_addr,
            vat_id: "VAT-EX-00000000".to_string(),
            website: "examplecorp.com".to_string(),
        },
        buyer: Buyer {
            name: "John Doe".to_string(),
            address: buyer_addr,
            email: "john.doe@example.com".to_string(),
        },
        payment_due: DateTime { year: 2025, month: 8, day: 15, hour: 0, minute: 0, second: 0 },
        delivery_date: DateTime { year: 2025, month: 7, day: 14, hour: 0, minute: 0, second: 0 },
        delivery_type: Some("Standard Shipping".to_string()),
        extra_info: vec![
            ("Order Reference".to_string(), "987654321".to_string()),
            ("Project".to_string(), "Example Project".to_string()),
        ],
        payment_type: Some("Bank Transfer".to_string()),
        payment_info: vec![
            (
                "IBAN".to_string(),
                "DE00 5001 0517 5407 3249 31".to_string(),
            ),
            ("BIC".to_string(), "INGDDEFFXXX".to_string()),
        ],
        products: vec![
            Product {
                description: "Rusty Widget with very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very long description".to_string(),
                units: 10,
                cost_per_unit: 9.99,
                tax_rate: 0.19,
                tax_exempt_reason: None,
            },
            Product {
                description: "Gadget Pro".to_string(),
                units: 5,
                cost_per_unit: 19.95,
                tax_rate: 0.07,
                tax_exempt_reason: Some("WOOW".to_string()),
            },
            Product {
                description: "Exported Item (Reverse Charge)".to_string(),
                units: 2,
                cost_per_unit: 100.0,
                tax_rate: 0.0,
                tax_exempt_reason: Some("Intra-EU reverse charge".to_string()),
            },
        ],
        locale: Locale::de,
    };

    let xml_string = invoice.to_xml().expect("Failed to generate XML");

    let payload = serde_json::json!({
        "contentToValidate": xml_string,
        "validationType": "ubl"
    });

    let client = reqwest::blocking::Client::new();
    let res = client
        .post("https://www.itb.ec.europa.eu/vitb/rest/invoice/api/validate")
        .header("accept", "application/json")
        .json(&payload)
        .send()
        .expect("Failed to connect to ITB API");

    // 4. Capture and Debug
    let status = res.status();
    let raw_body = res.text().expect("Failed to read response body");

    assert!(
        status.is_success(),
        "API returned error {}: {}",
        status,
        raw_body
    );

    let report: serde_json::Value = serde_json::from_str(&raw_body)
        .expect(&format!("Failed to parse JSON. Body was: {}", raw_body));

    let is_success = report["success"].as_bool().unwrap_or(false) || report["result"] == "SUCCESS";

    if !is_success {
        println!("Validation Report: {:#?}", report);

        if let Some(reports) = report["reports"].as_object() {
            for (key, val) in reports {
                println!("Report Category: {}", key);
                if let Some(items) = val["item"].as_array() {
                    for item in items {
                        if item["type"] == "ERROR" {
                            println!("  [!] {}", item["description"]);
                        }
                    }
                }
            }
        }
        panic!("XML failed validation rules.");
    }

    println!("Validation Passed Successfully!");
}
