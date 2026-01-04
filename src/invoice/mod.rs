use locale_rs::Locale;
use locale_rs::datetime_formats::DateTime;
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

pub struct Seller {
    pub name: String,
    pub address: String,
    pub vat_id: String,
    pub website: String,
}

pub struct Buyer {
    pub name: String,
    pub address: String,
    pub email: String,
}

pub struct Product {
    pub description: String,
    pub units: u32,
    pub cost_per_unit: f64,
    pub tax_rate: f64,
    pub tax_exempt_reason: Option<String>,
}

pub struct Invoice {
    pub number: String,
    pub date: DateTime,
    pub seller: Seller,
    pub buyer: Buyer,
    pub payment_due: DateTime,
    pub delivery_date: DateTime,
    pub delivery_type: Option<String>,
    pub extra_info: Vec<(String, String)>,
    pub payment_type: Option<String>,
    pub payment_info: Vec<(String, String)>,
    pub products: Vec<Product>,
    pub locale: Locale,
}

impl Invoice {
    pub fn calculate_summary(&self) -> (f64, BTreeMap<OrderedFloat<f64>, f64>, f64) {
        let mut subtotal = 0.0;
        let mut tax_totals = BTreeMap::new();
        for product in &self.products {
            let line_total = product.units as f64 * product.cost_per_unit;
            subtotal += line_total;
            if product.tax_rate > 0.0 {
                *tax_totals
                    .entry(OrderedFloat(product.tax_rate))
                    .or_insert(0.0) += line_total * product.tax_rate;
            }
        }
        let total = subtotal + tax_totals.values().sum::<f64>();
        (subtotal, tax_totals, total)
    }
}
