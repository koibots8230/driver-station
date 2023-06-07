use std::borrow::ToOwned;
use std::fmt::Error;
use tokio::net::UdpSocket;
use tokio::task;
use tokio::time::{self, Duration, Interval};

#[non_exhaustive]
struct Constants;

impl Constants {
    pub const FMS_IP: String = "10.0.100.5:1160".to_owned();
    pub const SIM_IP: String = "127.0.0.1".to_owned();
    pub const FMS_SOCKET: UdpSocket = UdpSocket::bind("10.0.100.5:1145").unwrap();
    pub const RIO_NORMAL_PORT: &'_ str = ":1110";
    pub const RIO_FMS_PORT: str = *":1115";
}

pub struct DriverStation {
    socket: Option<UdpSocket>,
    quit: bool,
    count: u16,
    fms_connected: bool,
}

impl Default for DriverStation {
    fn default() -> Self {
        Self {
            socket: None,
            quit: false,
            count: 0,
            fms_connected: false,
        }
    }
}

impl DriverStation {
    pub fn new(team: u16) -> DriverStation {
        let mut ds = DriverStation::default();
        ds.socket = UdpSocket::bind(team_number_to_ip(team) + Constants::RIO_NORMAL_PORT);
        return ds;
    }

    pub fn init() -> () {
        tokio::spawn(async {
            let mut update_rio: Interval = time::interval(Duration::from_millis(20));

            while !Self.quit {
                DriverStation::ds_to_rio().await;
                update_rio.tick().await;
            }
        });

        tokio::spawn(async {
            if Constants::FMS_SOCKET.connect(Constants::FMS_IP).await.is_ok() {
                Self.fms_connected = true;
            }

            if Self..connect(Constants::SIM_IP + Constants::RIO_NORMAL_PORT.as_str());

        });
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

