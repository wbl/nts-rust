#[derive(Debug, Copy, Clone)]
pub struct NTSKeys {
    pub c2s: [u8; 32],
    pub s2c: [u8; 32],
}
