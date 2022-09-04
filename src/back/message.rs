use super::stork_api::Stock;

#[derive(Debug)]
pub enum ToBackend {
    Refresh,
    SetInterval(u32),
    StockAdd(String),
    StockDel(String),
}

#[derive(Debug)]
pub enum ToFrontend {
    DataList(Vec<Stock>),
}


