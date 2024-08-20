use crate::treepp::*;

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

#[allow(non_snake_case)]
pub fn OP_256MUL() -> Script {
    #[cfg(feature = "assume-op-cat")]
    script! {
        OP_SIZE OP_NOT OP_NOTIF
        OP_PUSHBYTES_1 OP_PUSHBYTES_0 OP_SWAP OP_CAT
        OP_ENDIF
    }
    #[cfg(not(feature = "assume-op-cat"))]
    script! {
        OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD
        OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD OP_DUP OP_ADD
    }
}

#[allow(non_snake_case)]
pub fn OP_HINT() -> Script {
    script! {
        OP_DEPTH OP_1SUB OP_ROLL
    }
}
