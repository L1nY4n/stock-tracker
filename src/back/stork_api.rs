
use encoding_rs::GBK;
use lazy_static::lazy_static;
use regex::Regex;
const BASE_URL: &str = "http://hq.sinajs.cn";

const MIN_LEN: usize = "var hq_str_cc000000=\"\";".len();

lazy_static! {
    static ref REG: Regex = Regex::new(r"(^(sz|sh)\d{6}$)").unwrap();
}

pub type Vol =  i32;
pub type Price = f32;

#[derive(Clone, Default, Debug)]
pub struct Stock {
    pub name: String,
    pub code: String,
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
    pub bids: Vec<(Vol,Price)>,
    pub asks: Vec<(Vol,Price)>,
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

            let stocks =  str.trim()
            .split('\n')
            .filter(|x| x.len() > MIN_LEN)
            .filter_map(|stock_item_str| {decode_from_string(stock_item_str)}).collect();
            Ok(stocks)
        } else {
            Err(resp.status_text)
        }
    })
}

fn decode_from_string(stock_string: &str) -> Option<Stock> {

            let mut list: Vec<&str> = stock_string.trim().split(",").collect();
            list.truncate(32);
            // name
          
            match list.as_slice() {
                 [code_and_name,opening_str, closing, new_str, high, low, bid, ask, vol, amount,rest @ ..,date,time]
                 =>{
                    let name_str: Vec<&str> =code_and_name.split("=\"").collect();
                    let code = name_str[0].replace("var hq_str_", "");
                    let name = name_str[1];
                    let opening = opening_str.parse::<Price>().unwrap();
                    let new = new_str.parse::<Price>().unwrap();
                    let percent = ((new - opening) / opening * 10000.0).round() / 100.0;
                  
                   let bids =rest[0..10].chunks(2).into_iter().map(|x|{
                     if let [v,p] = x {
                        (v.parse::<Vol>().unwrap()/100,p.parse::<Price>().unwrap())
                      } else { 
                        (0,0.0)
                       }
                   }).collect();

                   let asks =rest[10..20].chunks(2).into_iter().map(|x|{
                    if let [v,p] = x {
                       (v.parse::<Vol>().unwrap()/100,p.parse::<Price>().unwrap())
                     } else { 
                       (0,0.0)
                      }
                  }).collect();
                
let stock =  Stock {
                name: String::from(name),
                code: String::from(code),
                opening,
                closing:closing.parse::<Price>().unwrap(),
                new,
                hight: high.parse::<Price>().unwrap(),
                low: low.parse::<Price>().unwrap(),
                bid : bid.parse::<Price>().unwrap(),
                ask : ask.parse::<Price>().unwrap(),
                vol : vol.parse::<Vol>().unwrap(),
                amount: amount.parse::<f32>().unwrap(),
                date: date.to_string(),
                time: time.to_string(),
               rise_per: percent,
                bids,
                asks,
                ..Default::default()
            };
            Some(stock)
                 }
                 _=> None
            }
        
             


          
        
        

           
   
}
