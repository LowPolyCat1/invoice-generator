use super::{Buyer, Product, Seller};
use locale_rs::Locale;
use locale_rs::datetime_formats::DateTime;
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

pub struct Invoice {
    pub number: String,
    pub date: DateTime,
    pub seller: Seller,
    pub buyer: Buyer,
    pub payment_due: DateTime,
    pub delivery_date: Option<DateTime>,
    pub delivery_type: Option<String>,
    pub extra_info: Option<Vec<(String, String)>>,
    pub payment_type: Option<String>,
    pub payment_info: Option<Vec<(String, String)>>,
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
