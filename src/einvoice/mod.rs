use crate::models::{Address, Invoice};
use locale_rs::datetime_formats::DateTime;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::collections::HashMap;
use std::io::Cursor;

pub fn to_ubl_date(dt: &DateTime) -> String {
    format!("{:04}-{:02}-{:02}", dt.year, dt.month, dt.day)
}

impl Invoice {
    pub fn to_xml(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 4);
        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        let mut root = BytesStart::new("Invoice");
        root.push_attribute((
            "xmlns",
            "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2",
        ));
        root.push_attribute((
            "xmlns:cac",
            "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2",
        ));
        root.push_attribute((
            "xmlns:cbc",
            "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2",
        ));
        writer.write_event(Event::Start(root))?;

        write_element(
            &mut writer,
            "cbc:CustomizationID",
            "urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0",
        )?;
        write_element(
            &mut writer,
            "cbc:ProfileID",
            "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0",
        )?;
        write_element(&mut writer, "cbc:ID", &self.number)?;
        write_element(&mut writer, "cbc:IssueDate", &to_ubl_date(&self.date))?;
        write_element(&mut writer, "cbc:DueDate", &to_ubl_date(&self.payment_due))?;
        write_element(&mut writer, "cbc:InvoiceTypeCode", "380")?;
        write_element(&mut writer, "cbc:DocumentCurrencyCode", "EUR")?;
        write_element(&mut writer, "cbc:BuyerReference", "Reference")?;

        self.write_supplier_party(&mut writer)?;

        self.write_customer_party(&mut writer)?;

        if let Some(delivery_date) = &self.delivery_date {
            writer.write_event(Event::Start(BytesStart::new("cac:Delivery")))?;
            write_element(
                &mut writer,
                "cbc:ActualDeliveryDate",
                &to_ubl_date(delivery_date),
            )?;
            writer.write_event(Event::End(BytesEnd::new("cac:Delivery")))?;
        }

        writer.write_event(Event::Start(BytesStart::new("cac:PaymentMeans")))?;
        write_element(&mut writer, "cbc:PaymentMeansCode", "30")?; // 30 = Bank transfer (SEPA)

        if let Some(payment_info) = &self.payment_info
            && !payment_info.is_empty() {
                let mut iban = String::new();
                let mut bic = String::new();

                for (key, value) in payment_info {
                    if key.to_uppercase() == "IBAN" {
                        iban = value.clone();
                    } else if key.to_uppercase() == "BIC" {
                        bic = value.clone();
                    }
                }

                if !iban.is_empty() {
                    writer
                        .write_event(Event::Start(BytesStart::new("cac:PayeeFinancialAccount")))?;
                    write_element(&mut writer, "cbc:ID", &iban)?;
                    if !bic.is_empty() {
                        writer.write_event(Event::Start(BytesStart::new(
                            "cac:FinancialInstitutionBranch",
                        )))?;
                        write_element(&mut writer, "cbc:ID", &bic)?;
                        writer.write_event(Event::End(BytesEnd::new(
                            "cac:FinancialInstitutionBranch",
                        )))?;
                    }
                    writer.write_event(Event::End(BytesEnd::new("cac:PayeeFinancialAccount")))?;
                }
            }

        writer.write_event(Event::End(BytesEnd::new("cac:PaymentMeans")))?;

        let mut sum_line_net = 0.0;
        let mut total_vat = 0.0;
        let mut vat_groups: HashMap<(String, String), (f64, f64, Vec<String>)> = HashMap::new();

        for prod in &self.products {
            let line_net = prod.units as f64 * prod.cost_per_unit;
            let category = if prod.tax_exempt_reason.is_some() {
                "E".to_string()
            } else {
                "S".to_string()
            };
            let rate = if category == "E" {
                0.0
            } else {
                prod.tax_rate * 100.0
            };
            let line_vat = (line_net * rate / 100.0 * 100.0).round() / 100.0;

            sum_line_net += line_net;
            total_vat += line_vat;

            let entry = vat_groups
                .entry((category, format!("{:.0}", rate)))
                .or_insert((0.0, 0.0, Vec::new()));
            entry.0 += line_net;
            entry.1 += line_vat;
            if let Some(r) = &prod.tax_exempt_reason {
                entry.2.push(r.clone());
            }
        }

        writer.write_event(Event::Start(BytesStart::new("cac:TaxTotal")))?;
        let mut t_amt = BytesStart::new("cbc:TaxAmount");
        t_amt.push_attribute(("currencyID", "EUR"));
        write_element_with_attr(&mut writer, t_amt, &format!("{:.2}", total_vat))?;

        for ((cat, rate), (taxable, tax, reasons)) in vat_groups {
            writer.write_event(Event::Start(BytesStart::new("cac:TaxSubtotal")))?;
            let mut t_able = BytesStart::new("cbc:TaxableAmount");
            t_able.push_attribute(("currencyID", "EUR"));
            write_element_with_attr(&mut writer, t_able, &format!("{:.2}", taxable))?;
            let mut t_sub_amt = BytesStart::new("cbc:TaxAmount");
            t_sub_amt.push_attribute(("currencyID", "EUR"));
            write_element_with_attr(&mut writer, t_sub_amt, &format!("{:.2}", tax))?;

            writer.write_event(Event::Start(BytesStart::new("cac:TaxCategory")))?;
            write_element(&mut writer, "cbc:ID", &cat)?;
            write_element(&mut writer, "cbc:Percent", &rate)?;
            if cat == "E" && !reasons.is_empty() {
                write_element(&mut writer, "cbc:TaxExemptionReason", &reasons.join(", "))?;
            }
            writer.write_event(Event::Start(BytesStart::new("cac:TaxScheme")))?;
            write_element(&mut writer, "cbc:ID", "VAT")?;
            writer.write_event(Event::End(BytesEnd::new("cac:TaxScheme")))?;
            writer.write_event(Event::End(BytesEnd::new("cac:TaxCategory")))?;
            writer.write_event(Event::End(BytesEnd::new("cac:TaxSubtotal")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("cac:TaxTotal")))?;

        writer.write_event(Event::Start(BytesStart::new("cac:LegalMonetaryTotal")))?;
        let totals = [
            ("cbc:LineExtensionAmount", sum_line_net),
            ("cbc:TaxExclusiveAmount", sum_line_net),
            ("cbc:TaxInclusiveAmount", sum_line_net + total_vat),
            ("cbc:PayableAmount", sum_line_net + total_vat),
        ];
        for (tag, val) in totals {
            let mut el = BytesStart::new(tag);
            el.push_attribute(("currencyID", "EUR"));
            write_element_with_attr(&mut writer, el, &format!("{:.2}", val))?;
        }
        writer.write_event(Event::End(BytesEnd::new("cac:LegalMonetaryTotal")))?;

        for (i, prod) in self.products.iter().enumerate() {
            let line_net = prod.units as f64 * prod.cost_per_unit;
            writer.write_event(Event::Start(BytesStart::new("cac:InvoiceLine")))?;
            write_element(&mut writer, "cbc:ID", &(i + 1).to_string())?;
            let mut qty = BytesStart::new("cbc:InvoicedQuantity");
            qty.push_attribute(("unitCode", "H87"));
            write_element_with_attr(&mut writer, qty, &prod.units.to_string())?;
            let mut l_ext = BytesStart::new("cbc:LineExtensionAmount");
            l_ext.push_attribute(("currencyID", "EUR"));
            write_element_with_attr(&mut writer, l_ext, &format!("{:.2}", line_net))?;
            writer.write_event(Event::Start(BytesStart::new("cac:Item")))?;
            write_element(&mut writer, "cbc:Name", &prod.description)?;
            writer.write_event(Event::Start(BytesStart::new("cac:ClassifiedTaxCategory")))?;
            let cat = if prod.tax_exempt_reason.is_some() {
                "E"
            } else {
                "S"
            };
            write_element(&mut writer, "cbc:ID", cat)?;
            write_element(
                &mut writer,
                "cbc:Percent",
                &format!(
                    "{:.0}",
                    if cat == "E" {
                        0.0
                    } else {
                        prod.tax_rate * 100.0
                    }
                ),
            )?;
            writer.write_event(Event::Start(BytesStart::new("cac:TaxScheme")))?;
            write_element(&mut writer, "cbc:ID", "VAT")?;
            writer.write_event(Event::End(BytesEnd::new("cac:TaxScheme")))?;
            writer.write_event(Event::End(BytesEnd::new("cac:ClassifiedTaxCategory")))?;
            writer.write_event(Event::End(BytesEnd::new("cac:Item")))?;
            writer.write_event(Event::Start(BytesStart::new("cac:Price")))?;
            let mut p_amt = BytesStart::new("cbc:PriceAmount");
            p_amt.push_attribute(("currencyID", "EUR"));
            write_element_with_attr(&mut writer, p_amt, &format!("{:.2}", prod.cost_per_unit))?;
            writer.write_event(Event::End(BytesEnd::new("cac:Price")))?;
            writer.write_event(Event::End(BytesEnd::new("cac:InvoiceLine")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("Invoice")))?;
        Ok(String::from_utf8(writer.into_inner().into_inner())?)
    }

    fn write_supplier_party(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        writer.write_event(Event::Start(BytesStart::new("cac:AccountingSupplierParty")))?;
        writer.write_event(Event::Start(BytesStart::new("cac:Party")))?;

        writer.write_event(Event::Start(BytesStart::new("cac:PartyName")))?;
        write_element(writer, "cbc:Name", &self.seller.name)?;
        writer.write_event(Event::End(BytesEnd::new("cac:PartyName")))?;

        write_address_block(writer, &self.seller.address)?;

        writer.write_event(Event::Start(BytesStart::new("cac:PartyTaxScheme")))?;
        write_element(writer, "cbc:CompanyID", &self.seller.vat_id)?;
        writer.write_event(Event::Start(BytesStart::new("cac:TaxScheme")))?;
        write_element(writer, "cbc:ID", "VAT")?;
        writer.write_event(Event::End(BytesEnd::new("cac:TaxScheme")))?;
        writer.write_event(Event::End(BytesEnd::new("cac:PartyTaxScheme")))?;

        writer.write_event(Event::Start(BytesStart::new("cac:PartyLegalEntity")))?;
        write_element(writer, "cbc:RegistrationName", &self.seller.name)?;
        writer.write_event(Event::End(BytesEnd::new("cac:PartyLegalEntity")))?;

        writer.write_event(Event::Start(BytesStart::new("cac:Contact")))?;
        write_element(writer, "cbc:Name", &self.seller.name)?;
        if let Some(phone) = &self.seller.phone {
            write_element(writer, "cbc:Telephone", phone)?;
        }
        if let Some(email) = &self.seller.email {
            write_element(writer, "cbc:ElectronicMail", email)?;
        }
        writer.write_event(Event::End(BytesEnd::new("cac:Contact")))?;

        writer.write_event(Event::End(BytesEnd::new("cac:Party")))?;
        writer.write_event(Event::End(BytesEnd::new("cac:AccountingSupplierParty")))?;
        Ok(())
    }

    fn write_customer_party(
        &self,
        writer: &mut Writer<Cursor<Vec<u8>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        writer.write_event(Event::Start(BytesStart::new("cac:AccountingCustomerParty")))?;
        writer.write_event(Event::Start(BytesStart::new("cac:Party")))?;
        writer.write_event(Event::Start(BytesStart::new("cac:PartyName")))?;
        write_element(writer, "cbc:Name", &self.buyer.name)?;
        writer.write_event(Event::End(BytesEnd::new("cac:PartyName")))?;

        write_address_block(writer, &self.buyer.address)?;

        writer.write_event(Event::Start(BytesStart::new("cac:PartyLegalEntity")))?;
        write_element(writer, "cbc:RegistrationName", &self.buyer.name)?;
        writer.write_event(Event::End(BytesEnd::new("cac:PartyLegalEntity")))?;

        writer.write_event(Event::Start(BytesStart::new("cac:Contact")))?;
        write_element(writer, "cbc:Name", &self.buyer.name)?;
        if !self.buyer.email.is_empty() {
            write_element(writer, "cbc:ElectronicMail", &self.buyer.email)?;
        }
        writer.write_event(Event::End(BytesEnd::new("cac:Contact")))?;

        writer.write_event(Event::End(BytesEnd::new("cac:Party")))?;
        writer.write_event(Event::End(BytesEnd::new("cac:AccountingCustomerParty")))?;
        Ok(())
    }
}

fn write_element(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    tag: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    writer.write_event(Event::Start(BytesStart::new(tag)))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}

fn write_element_with_attr(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    tag: BytesStart,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let name_bytes = tag.name().as_ref().to_vec();
    writer.write_event(Event::Start(tag))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(String::from_utf8(name_bytes)?)))?;
    Ok(())
}

fn write_address_block(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    addr: &Address,
) -> Result<(), Box<dyn std::error::Error>> {
    writer.write_event(Event::Start(BytesStart::new("cac:PostalAddress")))?;
    write_element(writer, "cbc:StreetName", &addr.street)?;
    write_element(writer, "cbc:CityName", &addr.town)?;
    write_element(writer, "cbc:PostalZone", &addr.code.to_string())?;
    writer.write_event(Event::Start(BytesStart::new("cac:Country")))?;
    write_element(writer, "cbc:IdentificationCode", "DE")?;
    writer.write_event(Event::End(BytesEnd::new("cac:Country")))?;
    writer.write_event(Event::End(BytesEnd::new("cac:PostalAddress")))?;
    Ok(())
}
