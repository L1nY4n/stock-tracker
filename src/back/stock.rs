use chrono::{NaiveDate, NaiveDateTime};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "http://hq.sinajs.cn";

const MIN_LEN: usize = "var hq_str_cc000000=\"\";".len();

static REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"...").unwrap());

pub type Vol = u64;
pub type Price = f32;

#[derive(Clone, Default, Debug)]
pub struct Stock {
    pub kline_scale: KLineScale,
    pub name: String,
    pub code: String,
    pub data: BaseData,

    pub klines: Vec<KlineItem>,

    pub show_klines_viewport: bool,
}

#[derive(Clone, Default, Debug)]
pub struct BaseData {
    pub date: String,
    pub time: String,
    pub opening: Price,
    pub closing: Price,
    pub hight: Price,
    pub low: Price,
    pub vol: Vol,
    pub amount: f32,
    pub bid: Price,
    pub ask: Price,
    pub new: Price,
    pub rise_per: f32,
    pub bids: Vec<(Vol, Price)>,
    pub asks: Vec<(Vol, Price)>,
}

// {
//     "day": "2024-09-25 10:45:00",
//     "open": "49.310",
//     "high": "52.900",
//     "low": "48.500",
//     "close": "52.130",
//     "volume": "2237250",
//     "amount": "113002687.0127"
//   }
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct KlineItemD {
    pub day: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub amount: Option<String>,
}

#[derive(Default, Debug, Clone)]
pub struct KlineItem {
    pub day: NaiveDateTime,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum KLineScale {
    Munute5,
    #[default]
    Munute15,
    Munute30,
    Hour,
    Day,
    Week,
    Month,
}

impl KLineScale {
    pub fn to_usize(&self) -> usize {
        match self {
            KLineScale::Munute5 => 5,
            KLineScale::Munute15 => 15,
            KLineScale::Munute30 => 30,
            KLineScale::Hour => 60,
            KLineScale::Day => 240,
            KLineScale::Week => 1200,
            KLineScale::Month => 60,
        }
    }
}

impl From<KlineItemD> for KlineItem {
    fn from(item: KlineItemD) -> Self {
        let day = match NaiveDateTime::parse_from_str(item.day.as_str(), "%Y-%m-%d %H:%M:%S") {
            Ok(d) => d,
            Err(_) => NaiveDate::parse_from_str(item.day.as_str(), "%Y-%m-%d")
                .unwrap()
                .into(),
        };

        Self {
            day,
            open: item.open.parse::<f64>().unwrap(),
            high: item.high.parse::<f64>().unwrap(),
            low: item.low.parse::<f64>().unwrap(),
            close: item.close.parse::<f64>().unwrap(),
            volume: item.volume.parse::<f64>().unwrap(),
            amount: item
                .amount
                .map(|x| x.parse::<f64>().unwrap())
                .unwrap_or(0.0),
        }
    }
}

pub fn check_stock_code(code: &str) -> bool {
    REG.is_match(code)
}

impl Stock {
    pub fn new(code: &str, name: &str) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn set_data(&mut self, data: BaseData) {
        self.data = data;
    }

    pub fn set_klines(&mut self, klines: Vec<KlineItem>) {
        self.klines = klines;
    }

    #[inline]
    pub fn data_new(&self) -> f32 {
        self.data.new
    }

    #[inline]
    pub fn data_close(&self) -> f32 {
        self.data.closing
    }

    #[inline]
    pub fn data_open(&self) -> f32 {
        self.data.opening
    }

    #[inline]
    pub fn data_rise_per(&self) -> f32 {
        self.data.rise_per
    }

    #[inline]
    pub fn data_hight(&self) -> f32 {
        self.data.hight
    }

    #[inline]
    pub fn data_low(&self) -> f32 {
        self.data.low
    }

    #[inline]
    pub fn data_vol(&self) -> f32 {
        self.data.vol as f32
    }

    #[inline]
    pub fn data_amount(&self) -> f32 {
        self.data.amount
    }

    #[inline]
    pub fn data_bids(&self) -> &Vec<(Vol, Price)> {
        &self.data.bids
    }

    #[inline]
    pub fn data_asks(&self) -> &Vec<(Vol, Price)> {
        &self.data.asks
    }

    pub fn get_kelines(
        code: &str,
        scale: &usize,
        datalen: u32,
    ) -> Result<Vec<KlineItem>, reqwest::Error> {
        let l =   reqwest::blocking::get(format!("https://quotes.sina.cn/cn/api/json_v2.php/CN_MarketDataService.getKLineData?symbol={code}&scale={scale}&ma=no&datalen={datalen}"))
        ?.json::<Vec<KlineItemD>>()?;
        let d = l.into_iter().map(KlineItem::from).collect();
        Ok(d)
    }
}

pub fn fetch_blocking(
    codes: Vec<String>,
) -> Result<Vec<(String, String, BaseData)>, reqwest::Error> {
    let code_string = codes.join(",");
    let url = format!("{}/list={}", BASE_URL, code_string);

    let str = reqwest::blocking::Client::new()
        .get(&url)
        .header("Referer", "https://www.sina.com.cn/")
        .send()?
        .text()?;

    let stocks = str
        .trim()
        .split('\n')
        .filter(|x| x.len() > MIN_LEN)
        .filter_map(|stock_item_str| decode_from_string(stock_item_str))
        .collect();
    Ok(stocks)
}

fn decode_from_string(stock_string: &str) -> Option<(String, String, BaseData)> {
    let mut list: Vec<&str> = stock_string.trim().split(",").collect();
    list.truncate(32);
    // name

    match list.as_slice() {
        [code_and_name, opening_str, closing_str, new_str, high, low, bid, ask, vol, amount, rest @ .., date, time] =>
        {
            let name_str: Vec<&str> = code_and_name.split("=\"").collect();
            let code = name_str[0].replace("var hq_str_", "");
            let name = name_str[1];
            let opening = opening_str.parse::<Price>().unwrap();
            let closing = closing_str.parse::<Price>().unwrap();
            let new = new_str.parse::<Price>().unwrap();
            let percent = ((new - closing) / closing * 10000.0).round() / 100.0;

            let bids = rest[0..10]
                .chunks(2)
                .into_iter()
                .map(|x| {
                    if let [v, p] = x {
                        (v.parse::<Vol>().unwrap() / 100, p.parse::<Price>().unwrap())
                    } else {
                        (0, 0.0)
                    }
                })
                .collect();

            let asks = rest[10..20]
                .chunks(2)
                .into_iter()
                .map(|x| {
                    if let [v, p] = x {
                        (v.parse::<Vol>().unwrap() / 100, p.parse::<Price>().unwrap())
                    } else {
                        (0, 0.0)
                    }
                })
                .collect();

            let data = BaseData {
                opening,
                closing,
                new,
                hight: high.parse::<Price>().unwrap(),
                low: low.parse::<Price>().unwrap(),
                bid: bid.parse::<Price>().unwrap(),
                ask: ask.parse::<Price>().unwrap(),
                vol: vol.parse::<Vol>().unwrap(),
                amount: amount.parse::<f32>().unwrap(),
                date: date.to_string(),
                time: time.to_string(),
                rise_per: percent,
                bids,
                asks,
            };
            Some((code.into(), name.into(), data))
        }
        _ => None,
    }
}
