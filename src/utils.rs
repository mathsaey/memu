use ggez::{graphics::*, *};

#[inline]
pub fn draw_pixel(ctx: &mut Context, x: usize, y: usize, col: Color) -> GameResult<()> {
    let rect = Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        [x as f32, y as f32, 1.0, 1.0].into(),
        col,
    )?;
    graphics::draw(ctx, &rect, DrawParam::default())?;
    Ok(())
}
