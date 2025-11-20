#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Instrument {
    pub id: u32,
    pub symbol: String,
    pub isin: String,
}
