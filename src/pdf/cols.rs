use printpdf::Mm;

pub fn col_pos(weights: Vec<i16>, page_width: Mm, margin_left: Mm, margin_right: Mm) -> Vec<Mm> {
    let total_weight: i16 = weights.iter().sum();
    let available_width = page_width.0 - margin_left.0 - margin_right.0;

    let mut positions = Vec::new();
    let mut current_x = margin_left.0;

    for weight in weights {
        positions.push(Mm(current_x));

        let col_width = (weight as f32 / total_weight as f32) * available_width;

        current_x += col_width;
    }

    positions.push(Mm(current_x));

    positions
}
