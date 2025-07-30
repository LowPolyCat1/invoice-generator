use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn compute_hmac(invoice_id: &str, pdf_bytes: &[u8], secret: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret)
        .expect("HMAC can take key of any size");
    mac.update(invoice_id.as_bytes());
    mac.update(pdf_bytes);
    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    hex::encode(code_bytes)
}
