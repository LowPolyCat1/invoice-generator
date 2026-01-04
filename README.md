# Rust Invoice Generator

A lightweight and efficient Rust library for generating professional PDF invoices. This library allows you to define sellers, buyers, and complex product lists (including tax rates and reverse charges) to produce clean, localized PDF documents.

## Features

* **PDF Generation:** Create high-quality PDFs with custom branding.
* **Tax Support:** Handles multiple tax rates and tax-exempt reasons.
* **Localization:** Support for different currencies and locale-specific formatting.
* **Flexible Layout:** Handles long product descriptions and multi-page items gracefully.

## Compliance Ready

This library is designed to meet the requirements for both **EU VAT/Reverse Charge** and **US Sales Tax** documentation.

* **EU Compliance:** Supports VAT IDs, Tax-Exempt reasons for Intra-Community supplies, and Reverse Charge notations.
* **US Compliance:** Supports Sales Tax line items, shipping/handling breakdown, and clear Nexus-appropriate addresses.

---

## Example Output

Below is an example of a generated invoice.

![Invoice Preview](./invoice.pdf)

---

---

## Usage

### First clone this repo

```shell
git clone <repo>
```

### Add the repo to your Cargo.toml

```toml
[dependencies]
invoice-gen = { path = <path> }
```

---

## Quick Start

```rust
use invoice::{Buyer, Invoice, Locale, Product, Seller, generate_invoice_pdf};
use std::fs::File;
use std::io::Write;

fn main() {
    let invoice = Invoice {
        number: "INV-2025-EXAMPLE".to_string(),
        date: "2025-07-15".to_string(),
        seller: Seller {
            name: "Example Corp".to_string(),
            address: "123 Main Street, 90210 Anytown".to_string(),
            vat_id: "VAT-EX-00000000".to_string(),
            website: "examplecorp.com".to_string(),
        },
        buyer: Buyer {
            name: "John Doe".to_string(),
            address: "456 Oak Avenue, 10001 Cityville".to_string(),
            email: "john.doe@example.com".to_string(),
        },
        payment_due: "2025-08-15".to_string(),
        delivery_date: "2025-07-14".to_string(),
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
                description: "Rusty Widget with long description...".to_string(),
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

    let pdf_bytes = generate_invoice_pdf(
        &invoice,
        "./fonts/OpenSans-Medium.ttf",
        Some("./res/logo.jpg"),
    )
    .expect("Failed to create PDF");

    let mut file = File::create("./invoice.pdf").expect("Unable to create output file");
    file.write_all(&pdf_bytes).expect("Failed to write PDF");

    println!("Invoice saved to 'invoice.pdf'");
}
```
