use std::{io};
use tempfile::NamedTempFile;
use crate::{
    format_currency, generate_invoice_pdf, save_pdf_bytes, Buyer, Invoice, Product, Seller
};

#[test]
fn test_format_currency() {
    assert_eq!(format_currency(0.0), "0, 00");
    assert_eq!(format_currency(1234.56), "1.234, 56");
    assert_eq!(format_currency(12.5), "12, 50");
}

#[test]
fn test_generate_invoice_pdf_in_memory() {
    let invoice = Invoice {
        number: "TEST-001".to_string(),
        date: "2025-07-30".to_string(),
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
        payment_due: "2025-08-15".to_string(),
        delivery_date: "2025-07-30".to_string(),
        delivery_type: None,
        extra_info: vec![],
        payment_type: None,
        payment_info: vec![],
        tax_rate: 0.19,
        products: vec![
            Product { description: "Widget".to_string(), units: 2, cost_per_unit: 10.0 },
            Product { description: "Gadget".to_string(), units: 1, cost_per_unit: 20.0 },
        ],
    };

    let pdf_bytes = generate_invoice_pdf(&invoice).unwrap();
    assert!(!pdf_bytes.is_empty(), "PDF bytes should not be empty");
}





#[test]
fn test_generate_and_save_pdf_tempfile() -> io::Result<()> {
    let invoice = Invoice {
        number: "TEST-002".to_string(),
        date: "2025-07-30".to_string(),
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
        payment_due: "2025-08-15".to_string(),
        delivery_date: "2025-07-30".to_string(),
        delivery_type: None,
        extra_info: vec![],
        payment_type: None,
        payment_info: vec![],
        tax_rate: 0.19,
        products: vec![
            Product { description: "Widget".to_string(), units: 2, cost_per_unit: 10.0 },
            Product { description: "Gadget".to_string(), units: 1, cost_per_unit: 20.0 },
        ],
    };

    let pdf_bytes = generate_invoice_pdf(&invoice).expect("Failed to generate PDF");

    let mut tmp_file = NamedTempFile::new()?;
    save_pdf_bytes(tmp_file.as_file_mut(), &pdf_bytes)?;

    let metadata = tmp_file.as_file().metadata()?;
    assert!(metadata.len() > 0, "Temporary PDF file is empty");

    Ok(())
}
