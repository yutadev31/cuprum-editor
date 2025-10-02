pub mod term;
pub mod vec2;

pub fn safe_add(x: usize, y: isize) -> usize {
    if y >= 0 {
        x.saturating_add(y as usize)
    } else {
        x.saturating_sub((-y) as usize)
    }
}
