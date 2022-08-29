use super::message::Stock;
use encoding_rs::GBK;
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
const BASE_URL: &str = "http://hq.sinajs.cn";
const LEN: usize = "var hq_str_cc000000=\"\";".len();
lazy_static! {
    static ref REG: Regex = Regex::new(r"(^(sz|sh)\d{6}$)").unwrap();

}

pub fn check_stock_code(code: &str) -> bool {
    REG.is_match(code)
}

pub fn fetch_blocking(codes: Vec<String>) -> Result<Vec<Stock>, ehttp::Error> {
    let headers = ehttp::headers(&[("referer", "http://finance.sina.com.cn")]);
    let code_string = codes.join(",");
    let url = format!("{}/list={}", BASE_URL, code_string);
    let request = ehttp::Request {
        url: url,
        method: "GET".to_string(),
        headers,
        body: vec![],
    };
    ehttp::fetch_blocking(&request).and_then(|resp| {
        if resp.ok {
            let (str, _, _) = GBK.decode(&resp.bytes);

            let stocks = decode_resp(str);
            Ok(stocks)
        } else {
            Err(resp.status_text)
        }
    })
}

fn decode_resp(stock_list_string: Cow<str>) -> Vec<Stock> {
    stock_list_string
        .trim()
        .split('\n')
        .filter(|x| x.len() > LEN)
        .map(|stock_string| {
            let list: Vec<&str> = stock_string.trim().split(",").collect();
            let name_str: Vec<&str> = list[0].split("=\"").collect();
            let code = name_str[0].replace("var hq_str_", "");
            let name = name_str[1];
            let date = list[30];
            let time = list[31];
            let base: f32 = list[2].parse().unwrap();
            let curr: f32 = list[3].parse().unwrap();
            let pes = (curr - base) / base * 100.0;
            let pes = (pes * 100.0).round() / 100.0;
            let buy1: i32 = list[10].parse().unwrap();
            let buy1 = buy1 / 100;

            Stock {
                name: String::from(name),
                code: String::from(code),
                date: date.to_string(),
                time: time.to_string(),
                percent: pes,
                curr: curr,
                buys: vec![buy1],
                ..Default::default()
            }
        })
        .collect()
}
