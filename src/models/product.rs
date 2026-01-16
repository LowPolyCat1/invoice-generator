pub struct Product {
    pub description: String,
    pub units: u32,
    pub cost_per_unit: f64,
    pub tax_rate: f64,
    pub tax_exempt_reason: Option<String>,
}
