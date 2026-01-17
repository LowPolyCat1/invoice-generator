use super::Address;

pub struct Seller {
    pub name: String,
    pub address: Address,
    pub vat_id: String,
    pub website: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}
