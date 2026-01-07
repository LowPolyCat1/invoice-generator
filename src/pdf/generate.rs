use crate::einvoice::{embed_facturx_xml, generate_cii_xml, inject_xmp_metadata};
use crate::invoice::*;
use crate::pdf::addresses::draw_address_section;
use crate::pdf::fin_summary::draw_financial_summary;
use crate::pdf::header::draw_header_info;
use crate::pdf::logo::draw_logo;
use crate::pdf::payment_details::draw_payment_details;
use crate::pdf::product_table::draw_product_table;
use crate::pdf::*;
use lopdf::{Dictionary, Object, Stream};
use printpdf::{Mm, PdfDocument, PdfPage, PdfSaveOptions};
use std::path::Path;

pub fn generate_invoice_pdf<P: AsRef<Path>>(
    invoice: &Invoice,
    font_path: P,
    logo_path: Option<P>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut doc = PdfDocument::new("Invoice");

    let font_bytes = std::fs::read(font_path)?;
    let font_id = doc.add_font(
        &ParsedFont::from_bytes(&font_bytes, 0, &mut Vec::new()).ok_or("Failed to parse font")?,
    );

    let mut ctx = PdfContext::new(font_id);
    let locale = &invoice.locale;

    let sanitize = |s: &str| s.replace(|c: char| c.is_control(), "");

    draw_logo(
        &mut ctx,
        logo_path.as_ref().map(|p| p.as_ref()),
        LEFT_MARGIN,
        &mut doc,
    )?;
    draw_header_info(&mut ctx, invoice);
    draw_address_section(&mut ctx, invoice);

    let (subtotal, tax_totals, total) = invoice.calculate_summary();
    draw_product_table(&mut ctx, invoice, &locale)?;

    let summary_top = ctx.y - Mm(8.0);
    draw_financial_summary(&mut ctx, &locale, subtotal, &tax_totals, total);
    draw_payment_details(&mut ctx, invoice, summary_top);

    ctx.pages.push(PdfPage::new(
        PAGE_WIDTH,
        PAGE_HEIGHT,
        ctx.current_ops.clone(),
    ));
    let pages = ctx.pages.clone();

    let pdf_bytes = doc
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new());

    let mut lopdf_doc = lopdf::Document::load_mem(&pdf_bytes)?;

    let xml_content = generate_cii_xml(invoice);
    let catalog_id = lopdf_doc.trailer.get(b"Root")?.as_reference()?;

    let all_objects = lopdf_doc.objects.keys().cloned().collect::<Vec<_>>();
    for id in all_objects {
        if let Ok(obj) = lopdf_doc.get_object(id) {
            if let Ok(dict) = obj.as_dict() {
                if dict
                    .get(b"Subtype")
                    .map_or(false, |s| s == &Object::Name("CIDFontType2".into()))
                {
                    let mut new_dict = dict.clone();
                    new_dict.set("CIDToGIDMap", Object::Name("Identity".into()));
                    lopdf_doc.set_object(id, Object::Dictionary(new_dict));
                }
            }
        }
    }

    let now_pdf = chrono::Utc::now().format("D:%Y%m%d%H%M%SZ").to_string();
    let info_dict = lopdf::dictionary! {
        "Title" => Object::String("Invoice".into(), lopdf::StringFormat::Literal),
        "Author" => Object::String(sanitize(&invoice.seller.name).into(), lopdf::StringFormat::Literal),
        "Producer" => Object::String("YourApp".into(), lopdf::StringFormat::Literal),
        "Creator" => Object::String("YourApp".into(), lopdf::StringFormat::Literal),
        "CreationDate" => Object::String(now_pdf.clone().into(), lopdf::StringFormat::Literal),
        "ModDate" => Object::String(now_pdf.into(), lopdf::StringFormat::Literal),
    };
    lopdf_doc.trailer.set("Info", Object::Dictionary(info_dict));

    inject_xmp_metadata(&mut lopdf_doc, catalog_id)?;
    embed_facturx_xml(&mut lopdf_doc, catalog_id, xml_content)?;

    add_output_intent(&mut lopdf_doc, catalog_id)?;

    let pages_id = lopdf_doc
        .get_object(catalog_id)?
        .as_dict()?
        .get(b"Pages")?
        .as_reference()?;
    let page_ids: Vec<lopdf::ObjectId> = lopdf_doc
        .get_object(pages_id)?
        .as_dict()?
        .get(b"Kids")?
        .as_array()?
        .iter()
        .filter_map(|obj| obj.as_reference().ok())
        .collect();

    for page_id in page_ids {
        let mut page = lopdf_doc.get_object(page_id)?.as_dict()?.clone();
        page.set(
            "Group",
            Object::Dictionary(lopdf::dictionary! {
                "Type" => "Group",
                "S" => "Transparency",
                "CS" => "DeviceRGB",
            }),
        );
        lopdf_doc.set_object(page_id, Object::Dictionary(page));
    }

    let mut out_buf = Vec::new();
    lopdf_doc.save_to(&mut out_buf)?;
    Ok(out_buf)
}

fn add_output_intent(
    doc: &mut lopdf::Document,
    catalog_id: lopdf::ObjectId,
) -> Result<(), Box<dyn std::error::Error>> {
    let icc_path = "assets/sRGB.icc";
    let icc_bytes = std::fs::read(icc_path)
        .map_err(|e| format!("Missing ICC profile at {}: {}", icc_path, e))?;

    let icc_stream = Stream::new(lopdf::dictionary! { "N" => 3 }, icc_bytes);
    let icc_id = doc.add_object(icc_stream);

    let output_intent = lopdf::dictionary! {
        "Type" => "OutputIntent",
        "S" => "GTS_PDFA1",
        "OutputConditionIdentifier" => Object::String("sRGB".into(), lopdf::StringFormat::Literal),
        "Info" => Object::String("sRGB IEC61966-2.1".into(), lopdf::StringFormat::Literal),
        "DestOutputProfile" => icc_id,
    };
    let intent_id = doc.add_object(output_intent);

    let mut catalog = doc.get_object(catalog_id)?.as_dict()?.clone();
    catalog.set(
        "OutputIntents",
        Object::Array(vec![Object::Reference(intent_id)]),
    );
    doc.set_object(catalog_id, Object::Dictionary(catalog));
    Ok(())
}

use lopdf::dictionary;
