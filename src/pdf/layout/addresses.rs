use crate::invoice::Invoice;
use crate::pdf::context::PdfContext;
use crate::pdf::{COL_1, COL_2};
use printpdf::Mm;

pub fn draw_address_section(ctx: &mut PdfContext, invoice: &Invoice) {
    ctx.y -= Mm(8.0);
    let col_width = (COL_2.0 - COL_1.0) - 5.0;

    let header_y = ctx.y;
    let seller_h_y = ctx.write_text_at_wrapping("Sold by", 15.0, COL_1, header_y, col_width);
    let buyer_h_y = ctx.write_text_at_wrapping("Billed to", 15.0, COL_2, header_y, col_width);
    ctx.y = seller_h_y.min(buyer_h_y);

    let name_row_y = ctx.y;
    let seller_n_y =
        ctx.write_text_at_wrapping(&invoice.seller.name, 10.0, COL_1, name_row_y, col_width);
    let buyer_n_y =
        ctx.write_text_at_wrapping(&invoice.buyer.name, 10.0, COL_2, name_row_y, col_width);
    ctx.y = seller_n_y.min(buyer_n_y) - Mm(1.0);

    let seller = format!(
        "{} {} {} {}",
        invoice.seller.address.street,
        invoice.seller.address.house_number,
        invoice.seller.address.code,
        invoice.seller.address.town
    );

    let buyer = format!(
        "{} {} {} {}",
        invoice.buyer.address.street,
        invoice.buyer.address.house_number,
        invoice.buyer.address.code,
        invoice.buyer.address.town
    );
    let mut seller_lines: std::str::Lines<'_> = seller.lines();
    let mut buyer_lines = buyer.lines();

    loop {
        let line_row_y = ctx.y;
        let s_line = seller_lines.next();
        let b_line = buyer_lines.next();

        if s_line.is_none() && b_line.is_none() {
            break;
        }

        let mut s_y = line_row_y;
        let mut b_y = line_row_y;

        if let Some(text) = s_line {
            s_y = ctx.write_text_at_wrapping(text, 9.0, COL_1, line_row_y, col_width);
        }
        if let Some(text) = b_line {
            b_y = ctx.write_text_at_wrapping(text, 9.0, COL_2, line_row_y, col_width);
        }

        ctx.y = s_y.min(b_y);
    }

    ctx.y -= Mm(1.0);
    let footer_row_y = ctx.y;
    let s_y = ctx.write_text_at_wrapping(
        &format!("VAT: {}", invoice.seller.vat_id),
        9.0,
        COL_1,
        footer_row_y,
        col_width,
    );
    let b_y = ctx.write_text_at_wrapping(&invoice.buyer.email, 9.0, COL_2, footer_row_y, col_width);

    ctx.y = s_y.min(b_y);
}
