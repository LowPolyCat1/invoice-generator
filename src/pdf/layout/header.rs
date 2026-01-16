use crate::invoice::Invoice;
use crate::pdf::context::PdfContext;
use crate::pdf::drawing::draw_line;
use crate::pdf::{COL_1, COL_2};
use printpdf::Mm;

pub fn draw_header_info(ctx: &mut PdfContext, invoice: &Invoice) {
    let col_width = (COL_2.0 - COL_1.0) - 5.0;

    ctx.y = ctx.write_text_at_wrapping(
        &format!("Invoice Id: {}", invoice.number),
        14.0,
        COL_1,
        ctx.y,
        col_width,
    );

    ctx.y -= Mm(4.0);

    let row_1_y = ctx.y;
    let left_y = ctx.write_text_at_wrapping(
        &invoice.locale.format_date(&invoice.date),
        10.0,
        COL_1,
        row_1_y,
        col_width,
    );
    let right_y = ctx.write_text_at_wrapping(
        &format!(
            "Payment Due: {}",
            &invoice.locale.format_date(&invoice.payment_due)
        ),
        10.0,
        COL_2,
        row_1_y,
        col_width,
    );
    ctx.y = left_y.min(right_y) - Mm(1.);

    let row_2_y = ctx.y;

    let mut extra_info_iter = invoice.extra_info.iter();

    if let Some((label, value)) = extra_info_iter.next() {
        let left_y = ctx.write_text_at_wrapping(
            &format!("{}: {}", label, value),
            10.0,
            COL_1,
            row_2_y,
            col_width,
        );
        let right_y = ctx.write_text_at_wrapping(
            &format!(
                "Delivery Date: {}",
                &invoice.locale.format_date(&invoice.delivery_date)
            ),
            10.0,
            COL_2,
            row_2_y,
            col_width,
        );
        ctx.y = left_y.min(right_y) - Mm(1.);
    } else {
        ctx.y = ctx.write_text_at_wrapping(
            &format!(
                "Delivery Date: {}",
                &invoice.locale.format_date(&invoice.delivery_date)
            ),
            10.0,
            COL_2,
            row_2_y,
            col_width,
        );
    }

    while let Some((l1, v1)) = extra_info_iter.next() {
        let current_row_y = ctx.y;

        let left_y = ctx.write_text_at_wrapping(
            &format!("{}: {}", l1, v1),
            10.0,
            COL_1,
            current_row_y,
            col_width,
        );

        let mut right_y = left_y;
        if let Some((l2, v2)) = extra_info_iter.next() {
            right_y = ctx.write_text_at_wrapping(
                &format!("{}: {}", l2, v2),
                10.0,
                COL_2,
                current_row_y,
                col_width,
            );
        }

        ctx.y = left_y.min(right_y) - Mm(1.0);
    }

    draw_line(&mut ctx.current_ops, COL_1, Mm(190.0), ctx.y);
}
