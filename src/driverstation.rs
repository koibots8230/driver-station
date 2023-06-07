pub mod state;

use std::{borrow::ToOwned, sync::Arc};
use bytes::Bytes;
use local_ip_address::local_ip;
use tokio::{runtime::{Builder, Runtime}, sync::Mutex, net::UdpSocket};

use self::state::DriverStationState;

type PacketShare = Arc<Mutex<[Bytes; 4]>>;

pub struct DriverStation {
    team_number: u16,
    ds: Runtime,
    udp_socket: UdpSocket,
}

impl DriverStation {
    pub async fn new(team: u16) -> Self {
        return Self {
            team_number: team,
            ds: Builder::new_multi_thread()
                .worker_threads(2)
                .thread_name("driver-station")
                .build()
                .unwrap(),
            udp_socket: UdpSocket::bind(local_ip().expect("Failed to retrieve localhost ip").to_string()).await.expect("Error: Failed to bind socket"),        }
    }

    pub fn get_rio_udp(self, state: &mut DriverStationState) {
        self.udp_socket.send_to(buf, target)
    }
}

// NOTE: Produces ip addresses with leading zeros. This should NOT be a problem according to the rust docs. https://doc.rust-lang.org/std/net/struct.Ipv4Addr.html#textual-representation
pub fn team_number_to_ip(team_number: u16) -> String {
    let mut tn = team_number.to_string();
    tn = "0".repeat(4 - tn.len()) + &tn;
    return "10.".to_owned() + &tn[0..2] + "." + &tn[2..4] + ".2";
}

