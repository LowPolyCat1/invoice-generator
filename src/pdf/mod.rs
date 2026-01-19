pub mod generate;

pub use generate::generate_invoice_pdf;

pub const LEFT_MARGIN: f32 = 20.0;
pub const COL_1: f32 = LEFT_MARGIN;
pub const COL_2: f32 = 120.0;
pub const PAGE_WIDTH: f32 = 210.0;
pub const PAGE_HEIGHT: f32 = 297.0;
pub const BOTTOM_MARGIN: f32 = 15.0;
