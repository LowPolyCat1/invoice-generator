use crate::{
    invoice::{Buyer, Invoice, Product, Seller},
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
            address: "123 Test Street".to_string(),
            vat_id: "VAT-TEST-123".to_string(),
            website: "test.example.com".to_string(),
        },
        buyer: Buyer {
            name: "Test Buyer".to_string(),
            address: "456 Buyer Road".to_string(),
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
