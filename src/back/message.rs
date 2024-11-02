use super::stock::{BaseData, KLineScale, KlineItem, Stock};

#[derive(Debug)]
pub enum ToBackend {
    Refresh,
    SetInterval(u32),
    StockAdd(String),
    StockDel(String),
    StockKLine(String, KLineScale)
}

#[derive(Debug)]
pub enum ToFrontend {
    DataList(Vec<(String,String,BaseData)>),
    Data(String,String,BaseData),
    Kline(String,Vec<KlineItem>)
}
