use ::invoice::Locale;
use ::invoice::models::*;
use ::invoice::pdf::generate_invoice_pdf;
use locale_rs::datetime_formats::DateTime;
use std::fs::File;
use std::io::Write;

fn main() {
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
            phone: Some("+49 123 456789".to_string()),
            email: Some("contact@examplecorp.com".to_string()),
        },
        buyer: Buyer {
            name: "John Doe".to_string(),
            address: buyer_addr,
            email: "john.doe@example.com".to_string(),
        },
        payment_due: DateTime { year: 2025, month: 8, day: 15, hour: 0, minute: 0, second: 0 },
        delivery_date: None,
        // Some(DateTime { year: 2025, month: 7, day: 14, hour: 0, minute: 0, second: 0 }),
        delivery_type: None,
        // Some("Standard Shipping".to_string()),
        extra_info: None,
        // Some(vec![
        //     ("Order Reference".to_string(), "987654321".to_string()),
        //     ("Project".to_string(), "Example Project".to_string()),
        // ]),
        payment_type: None,
        //  Some("Bank Transfer".to_string()),
        payment_info: Some(vec![
            (
                "IBAN".to_string(),
                "DE00 5001 0517 5407 3249 31".to_string(),
            ),
            ("BIC".to_string(), "INGDDEFFXXX".to_string()),
        ]),
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

    let xml_output = invoice.to_xml().expect("Failed to generate XML");

    let mut xml_file = File::create("./invoice.xml").expect("Unable to create XML file");
    xml_file
        .write_all(xml_output.as_bytes())
        .expect("Failed to write XML");

    println!("Invoice XML saved to 'invoice.xml'");

    let pdf_bytes = generate_invoice_pdf(
        &invoice,
        "./fonts/OpenSans-Medium.ttf",
        Some("./res/logo.jpg"),
    )
    .expect("Failed to create PDF");

    let mut file = File::create("./invoice.pdf").expect("Unable to create output file");
    file.write_all(&pdf_bytes).expect("Failed to write PDF");

    println!("Invoice saved to 'invoice.pdf' as PDF/A-3 with embedded ZUGFeRD data");
}
