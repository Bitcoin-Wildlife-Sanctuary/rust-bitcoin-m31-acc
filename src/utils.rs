pub fn m31_to_limbs(v: u32) -> [i32; 4] {
    [
        (v & 255) as i32,
        ((v >> 8) & 255) as i32,
        ((v >> 16) & 255) as i32,
        ((v >> 24) & 255) as i32,
    ]
}

pub fn cm31_to_limbs(real: u32, imag: u32) -> [i32; 8] {
    let real_limbs = m31_to_limbs(real);
    let imag_limbs = m31_to_limbs(imag);

    [
        real_limbs[0],
        real_limbs[1],
        real_limbs[2],
        real_limbs[3],
        imag_limbs[0],
        imag_limbs[1],
        imag_limbs[2],
        imag_limbs[3],
    ]
}
