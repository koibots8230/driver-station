use std::borrow::ToOwned;
use std::fmt::Error;
use tokio::net::UdpSocket;
use tokio::task::{self, JoinHandle};
use tokio::time::{self, Duration, Interval};

pub struct DriverStation {
    team_number: u16,
    socket: Option<UdpSocket>,
    quit: bool,
    count: u16,
    fms_connected: bool,
    connection: Option<JoinHandle<()>>,
}

impl Default for DriverStation {
    fn default() -> Self {
        Self {
            team_number: 0,
            socket: None,
            quit: false,
            count: 0,
            fms_connected: false,
            connection: None,
        }
    }
}

impl DriverStation {
    pub fn new(team: u16) -> Self {
        let mut ds = Self::default();
        ds.team_number = team;
        return ds;
    }

    pub fn init(self) -> Self {
        tokio::spawn(async {
            let mut update_rio: Interval = time::interval(Duration::from_millis(20));

            loop {
                DriverStation::ds_to_rio().await;
                update_rio.tick().await;
            }
        });

        tokio::spawn(async {

        });

        return self;
    }

    async fn ds_to_rio() {

    }

    fn ds_to_fms() {

    }
}

// NOTE: Produces ip addresses with leading zeros. This should NOT be a problem according to the rust docs. https://doc.rust-lang.org/std/net/struct.Ipv4Addr.html#textual-representation
pub fn team_number_to_ip(team_number: u16) -> String {
    let mut tn = team_number.to_string();
    tn = "0".repeat(4 - tn.len()) + &tn;
    return "10.".to_owned() + &tn[0..2] + "." + &tn[2..4] + ".2";
}

