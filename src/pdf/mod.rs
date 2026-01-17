use printpdf::*;

pub mod context;
pub mod drawing;
pub mod embed;
pub mod generate;
pub mod layout;
pub mod pdfa;

pub use context::PdfContext;
pub use drawing::{draw_line, draw_v_line};
pub use embed::embed_xml_in_pdf;
pub use generate::generate_invoice_pdf;
pub use pdfa::convert_to_pdfa3;

const LEFT_MARGIN: f32 = 20.0;
pub const COL_1: Mm = Mm(LEFT_MARGIN);
pub const COL_2: Mm = Mm(120.0);
pub const PAGE_WIDTH: Mm = Mm(210.0);
pub const PAGE_HEIGHT: Mm = Mm(297.0);
pub const BOTTOM_MARGIN: Mm = Mm(15.0);
