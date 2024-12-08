use rand::{thread_rng, Rng};

pub fn position(size: &(u16, u16)) -> (f32, f32) {
    let mut rng = thread_rng();
    let (w, h) = *size;
    let half_w = w as f32 / 2.0;
    let half_h = h as f32 / 2.0;
    (
        rng.gen_range(-half_w..=half_w),
        rng.gen_range(-half_h..=half_h),
    )
}

pub fn heading() -> u16 {
    thread_rng().gen_range(0..360)
}
