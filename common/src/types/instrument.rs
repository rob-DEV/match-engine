#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Instrument {
    pub id: u32,
    pub symbol: String,
    pub isin: String,
}

impl Instrument {
    pub fn instrument_str_to_fixed_buffer(symbol: &str) -> [u8; 16] {
        let bytes = symbol.as_bytes();
        let mut buf = [0u8; 16];
        let n = bytes.len().min(16);
        buf[..n].copy_from_slice(&bytes[..n]);
        buf
    }
}
