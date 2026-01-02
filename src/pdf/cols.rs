use printpdf::Mm;

pub fn col_pos(weights: Vec<i16>, page_width: Mm, margin_left: Mm, margin_right: Mm) -> Vec<Mm> {
    let total_weight: i16 = weights.iter().sum();
    let available_width = page_width.0 - margin_left.0 - margin_right.0;

    let mut positions = Vec::new();
    let mut current_x = margin_left.0;

    for weight in weights {
        // Push the starting X coordinate of the current column
        positions.push(Mm(current_x));

        // Calculate the width of this column based on its weight
        let col_width = (weight as f32 / total_weight as f32) * available_width;

        // Advance current_x for the next column
        current_x += col_width;
    }

    positions.push(Mm(current_x));

    positions
}
