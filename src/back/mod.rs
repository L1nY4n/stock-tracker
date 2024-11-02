use std::time::Duration;

use crossbeam::{
    channel::{tick, Receiver, Sender},
    select,
};
use eframe::egui::ahash::HashMap;
use stock::{KLineScale, Stock};
use tracing::error;

pub mod message;
use message::{ToBackend, ToFrontend};

use self::stock::check_stock_code;

pub mod stock;

#[derive(Debug, Clone)]
pub struct Back {
    stock_codes: Vec<String>,
    kline_scale_map: HashMap<String, KLineScale>,
    back_tx: Sender<ToFrontend>,
    front_rx: Receiver<ToBackend>,
}

impl Back {
    pub fn new(back_tx: Sender<ToFrontend>, front_rx: Receiver<ToBackend>, codes: String) -> Self {
        let stock_codes = codes
            .split(",")
            .filter(|x| check_stock_code(x))
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        Self {
            back_tx,
            front_rx,
            stock_codes,
            kline_scale_map: HashMap::default(),
        }
    }

    pub fn run(&mut self) {
        self.refetch_data();
        self.refresh_kline();
        let mut ticker = tick(Duration::from_millis(200));
        let kline_ticker = tick(Duration::from_secs(60));
        loop {
            select! {
                recv(self.front_rx)->msg =>{
                     match msg {
                         Ok(m)=>{
                                match m {
                                    ToBackend::Refresh=>{
                                        self.refetch_data();
                                    },
                                    ToBackend::SetInterval(interval) => {
                                            ticker = tick(Duration::from_millis(interval.into()));
                                        },
                                    ToBackend::StockAdd(code) => {
                                                println!("add code {}",code);
                                                self.add_stock(code);
                                    },
                                  ToBackend::StockDel(code) => {
                                        self.stock_codes.retain(|x| x != &code);
                                        self.refetch_data();

                                    },
                                    ToBackend::StockKLine(code, scale) => {
                                        let scale_int = scale.to_usize();
                                        self.kline_scale_map.insert(code.clone(), scale);
                                       match  Stock::get_kelines(&code, &scale_int, 100) {
                                           Ok(l) => {

                                               let dl = ToFrontend::Kline(code.clone(), l);
                                               self.back_tx.send(dl).ok();
                                           },
                                           Err(err) =>{
                                                println!("kline error {}",err);
                                           },
                                       }

                                        }
                                    }}
                         Err(e) => {
                                error!("receive ToBackend msg faild : {}",e)
                          }
                        }
                    },
                recv(kline_ticker)->_msg =>{

                    let this = self.clone();
                    std::thread::spawn(move || {
                        this.refresh_kline();
                    });

                },
                 recv(ticker)->_msg =>{
                   self.refetch_data();
                },
            }
        }
    }

    fn add_stock(&mut self, code: String) {
        if !self.stock_codes.contains(&code) {
            match stock::fetch_blocking(vec![code.to_string()]) {
                Ok(datas) => {
                    datas.iter().for_each(|(code, name, data)| {
                        let dl = ToFrontend::Data(code.clone(), name.clone(), data.clone());
                        self.back_tx.send(dl).ok();
                        self.stock_codes.push(code.clone());

                        let scale = self
                            .kline_scale_map
                            .get(code)
                            .unwrap_or(&KLineScale::Munute15);
                        match Stock::get_kelines(code, &scale.to_usize(), 100) {
                            Ok(kl) => {
                                self.back_tx.send(ToFrontend::Kline(code.clone(), kl)).ok();
                            }
                            Err(err) => {
                                println!("get kline error {}", err);
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("add stock  error {}", e)
                }
            }
        } else {
            println!("stock {} already exists", code);
        }
    }

    fn refetch_data(&self) {
        if !self.stock_codes.is_empty() {
            match stock::fetch_blocking(self.stock_codes.clone()) {
                Ok(datas) => {
                    let dl = ToFrontend::DataList(datas);
                    self.back_tx.send(dl).ok();
                }
                Err(e) => {
                    error!("fetch data error {}", e)
                }
            }
        }
    }

    fn refresh_kline(&self) {
        if !self.stock_codes.is_empty() {
            self.stock_codes.iter().for_each(|code| {
                let scale = self
                    .kline_scale_map
                    .get(code)
                    .unwrap_or(&KLineScale::Munute15);
                match Stock::get_kelines(code, &scale.to_usize(), 100) {
                    Ok(kl) => {
                        self.back_tx.send(ToFrontend::Kline(code.clone(), kl)).ok();
                    }
                    Err(err) => {
                        println!("get kline error {}", err);
                    }
                }
            });
        }
    }
}
