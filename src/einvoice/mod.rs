use crate::invoice::*;
use chrono::Utc;
use lopdf::{Dictionary, Object, Stream};

pub fn inject_xmp_metadata(
    doc: &mut lopdf::Document,
    catalog_id: lopdf::ObjectId,
) -> Result<(), Box<dyn std::error::Error>> {
    let xmp_content = r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
    <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description rdf:about="" xmlns:fx="http://www.factur-x.eu/import-models/en16931#">
            <fx:DocumentType>INVOICE</fx:DocumentType>
            <fx:DocumentFileName>factur-x.xml</fx:DocumentFileName>
            <fx:Version>1.0</fx:Version>
            <fx:ConformanceLevel>EN 16931</fx:ConformanceLevel>
        </rdf:Description>
    </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#
        .trim();

    let mut dict = Dictionary::new();
    dict.set("Type", Object::Name("Metadata".into()));
    dict.set("Subtype", Object::Name("XML".into()));
    let stream = Stream::new(dict, xmp_content.as_bytes().to_vec());
    let metadata_id = doc.add_object(stream);

    let mut catalog = doc.get_object(catalog_id)?.as_dict()?.clone();
    catalog.set("Metadata", Object::Reference(metadata_id));
    doc.set_object(catalog_id, Object::Dictionary(catalog));
    Ok(())
}

pub fn embed_facturx_xml(
    doc: &mut lopdf::Document,
    catalog_id: lopdf::ObjectId,
    xml_data: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().format("%Y%m%d%H%M%S+00'00'").to_string();
    let xml_bytes = xml_data.as_bytes().to_vec();

    let mut ef_stream_dict = Dictionary::new();
    ef_stream_dict.set("Type", Object::Name("EmbeddedFile".into()));
    ef_stream_dict.set("Subtype", Object::Name("text/xml".into()));
    ef_stream_dict.set("Length", Object::Integer(xml_bytes.len() as i64));
    ef_stream_dict.set(
        "Params",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "ModDate",
            Object::String(now.into(), lopdf::StringFormat::Literal),
        )])),
    );
    let stream_id = doc.add_object(Stream::new(ef_stream_dict, xml_bytes));

    let mut file_spec = Dictionary::new();
    file_spec.set("Type", Object::Name("Filespec".into()));
    file_spec.set(
        "F",
        Object::String("factur-x.xml".into(), lopdf::StringFormat::Literal),
    );
    file_spec.set(
        "UF",
        Object::String("factur-x.xml".into(), lopdf::StringFormat::Literal),
    );
    file_spec.set("AFRelationship", Object::Name("Data".into()));
    let mut ef_ref_dict = Dictionary::new();
    ef_ref_dict.set("F", Object::Reference(stream_id));
    file_spec.set("EF", Object::Dictionary(ef_ref_dict));
    let file_spec_id = doc.add_object(file_spec);

    let mut catalog = doc.get_object(catalog_id)?.as_dict()?.clone();
    catalog.set("AF", Object::Array(vec![Object::Reference(file_spec_id)]));
    let mut names_dict = Dictionary::new();
    let mut ef_names = Dictionary::new();
    ef_names.set(
        "Names",
        Object::Array(vec![
            Object::String("factur-x.xml".into(), lopdf::StringFormat::Literal),
            Object::Reference(file_spec_id),
        ]),
    );
    names_dict.set("EmbeddedFiles", Object::Dictionary(ef_names));
    catalog.set("Names", Object::Dictionary(names_dict));

    doc.set_object(catalog_id, Object::Dictionary(catalog));
    Ok(())
}

pub fn generate_cii_xml(invoice: &Invoice, subtotal: f64, total: f64) -> String {
    let issue_date = format!(
        "{}{:02}{:02}",
        invoice.date.year, invoice.date.month, invoice.date.day
    );
    let tax_amount = total - subtotal;

    let mut items_xml = String::new();
    for (idx, p) in invoice.products.iter().enumerate() {
        items_xml.push_str(&format!(
            r#"        <ram:IncludedSupplyChainTradeLineItem>
            <ram:AssociatedDocumentLineDocument><ram:LineID>{}</ram:LineID></ram:AssociatedDocumentLineDocument>
            <ram:SpecifiedTradeProduct><ram:Name>{}</ram:Name></ram:SpecifiedTradeProduct>
            <ram:SpecifiedLineTradeAgreement>
                <ram:NetPriceProductTradePrice><ram:ChargeAmount>{:.2}</ram:ChargeAmount></ram:NetPriceProductTradePrice>
            </ram:SpecifiedLineTradeAgreement>
            <ram:SpecifiedLineTradeDelivery><ram:BilledQuantity unitCode="C62">{}</ram:BilledQuantity></ram:SpecifiedLineTradeDelivery>
            <ram:SpecifiedLineTradeSettlement>
                <ram:ApplicableTradeTax>
                    <ram:TypeCode>VAT</ram:TypeCode>
                    <ram:CategoryCode>S</ram:CategoryCode>
                    <ram:RateApplicablePercent>{:.2}</ram:RateApplicablePercent>
                </ram:ApplicableTradeTax>
                <ram:SpecifiedTradeSettlementLineMonetarySummation>
                    <ram:LineTotalAmount>{:.2}</ram:LineTotalAmount>
                </ram:SpecifiedTradeSettlementLineMonetarySummation>
            </ram:SpecifiedLineTradeSettlement>
        </ram:IncludedSupplyChainTradeLineItem>
"#,
            idx + 1, p.description, p.cost_per_unit, p.units, p.tax_rate * 100.0, (p.units as f64 * p.cost_per_unit)
        ));
    }

    format!(
r#"<?xml version="1.0" encoding="UTF-8"?>
<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"
    xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100"
    xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100">
    <rsm:ExchangedDocumentContext>
        <ram:GuidelineSpecifiedDocumentContextParameter>
            <ram:ID>urn:cen.eu:en16931:2017#compliant#factur-x.eu:1p0:en16931</ram:ID>
        </ram:GuidelineSpecifiedDocumentContextParameter>
    </rsm:ExchangedDocumentContext>
    <rsm:ExchangedDocument>
        <ram:ID>{}</ram:ID>
        <ram:TypeCode>380</ram:TypeCode>
        <ram:IssueDateTime><udt:DateTimeString format="102">{}</udt:DateTimeString></ram:IssueDateTime>
    </rsm:ExchangedDocument>
    <rsm:SupplyChainTradeTransaction>
{}        <ram:ApplicableHeaderTradeAgreement>
            <ram:SellerTradeParty><ram:Name>{}</ram:Name></ram:SellerTradeParty>
            <ram:BuyerTradeParty><ram:Name>{}</ram:Name></ram:BuyerTradeParty>
        </ram:ApplicableHeaderTradeAgreement>
        <ram:ApplicableHeaderTradeDelivery/>
        <ram:ApplicableHeaderTradeSettlement>
            <ram:InvoiceCurrencyCode>EUR</ram:InvoiceCurrencyCode>
            <ram:ApplicableTradeTax>
                <ram:CalculatedAmount>{:.2}</ram:CalculatedAmount>
                <ram:TypeCode>VAT</ram:TypeCode>
                <ram:BasisAmount>{:.2}</ram:BasisAmount>
                <ram:CategoryCode>S</ram:CategoryCode>
                <ram:RateApplicablePercent>19.00</ram:RateApplicablePercent>
            </ram:ApplicableTradeTax>
            <ram:SpecifiedTradeSettlementHeaderMonetarySummation>
                <ram:LineTotalAmount>{:.2}</ram:LineTotalAmount>
                <ram:TaxBasisTotalAmount>{:.2}</ram:TaxBasisTotalAmount>
                <ram:TaxTotalAmount currencyID="EUR">{:.2}</ram:TaxTotalAmount>
                <ram:GrandTotalAmount>{:.2}</ram:GrandTotalAmount>
                <ram:DuePayableAmount>{:.2}</ram:DuePayableAmount>
            </ram:SpecifiedTradeSettlementHeaderMonetarySummation>
        </ram:ApplicableHeaderTradeSettlement>
    </rsm:SupplyChainTradeTransaction>
</rsm:CrossIndustryInvoice>"#,
        invoice.number,
        issue_date,
        items_xml,
        invoice.seller.name,
        invoice.buyer.name,
        tax_amount,
        subtotal,
        subtotal,
        subtotal,
        tax_amount,
        total,
        total
    ).trim().to_string()
}
