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

#[derive(Clone, Default, Debug)]
pub struct Stock {
    pub name: String,
    pub code: String,
    pub date: String,
    pub time: String,
    pub curr: f32,
    pub percent: f32,
    pub buys: Vec<i32>,
    pub sells: Vec<i32>,
}
