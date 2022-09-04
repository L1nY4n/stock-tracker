use std::time::Duration;

use crossbeam::{
    channel::{tick, Receiver, Sender},
    select,
};
use tracing::{error, info};

pub mod message;
use message::{ToBackend, ToFrontend};

use self::stork_api::check_stock_code;

pub mod stork_api;

pub struct Back {
    stock_codes: Vec<String>,
    back_tx: Sender<ToFrontend>,
    front_rx: Receiver<ToBackend>,
}

impl Back {
    pub fn new(back_tx: Sender<ToFrontend>, front_rx: Receiver<ToBackend>, codes: String) -> Self {
        let stock_codes = codes
            .split(",")
            .filter(|x|check_stock_code(x))
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        Self {
            back_tx,
            front_rx,
            stock_codes,
        }
    }

    pub fn init(&mut self) {
        self.refetch(false);
        #[warn(unused_assignments)]
       let mut ticker = tick(Duration::from_secs(5));
        loop {
            select! {
                recv(self.front_rx)->msg =>{
                     match msg {
                         Ok(m)=>{
                                match m {
                                    ToBackend::Refresh=>{
                                        self.refetch(false);
                                    },
                                    ToBackend::SetInterval(interval) => {
                                            ticker = tick(Duration::from_secs(interval.into()));
                                        },
                                    ToBackend::StockAdd(code) => {
                                        //   if self.stock_codes.contains(&code){
                                              self.stock_codes.push(code);
                                     //      }

                                    },
                                  ToBackend::StockDel(code) => {
                                        self.stock_codes.retain(|x| x != &code);
                                        self.refetch(true);
                                    }
                                     }
                                        }
                         Err(e) => {
                                error!("receive ToBackend msg faild : {}",e)
                          }
                        }
                    },
                 recv(ticker)->_msg =>{
                    info!("tk");
                   self.refetch(false);
                },
            }
        }
    }

    fn refetch(&self, focus: bool) {
        if !self.stock_codes.is_empty() {
            match stork_api::fetch_blocking(self.stock_codes.clone()) {
                Ok(stocks) => {
                    let dl = ToFrontend::DataList(stocks);
                    self.back_tx.send(dl).ok();
                }
                Err(e) => {
                    error!(e)
                }
            }
        }else{
            if focus {
                self.back_tx.send(ToFrontend::DataList(vec![])).ok();
            }
        }

      
    }
}
