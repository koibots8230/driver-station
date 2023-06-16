use std::mem::size_of;
use std::net::UdpSocket;

// NOTE: Produces ip addresses with leading zeros. This should NOT be a problem according to the rust docs. https://doc.rust-lang.org/std/net/struct.Ipv4Addr.html#textual-representation
pub fn team_number_to_ip(team_number: u16) -> String {
    let mut tn = team_number.to_string();
    tn = "0".repeat(4 - tn.len()) + &tn;
    return "10.".to_owned() + &tn[0..2] + "." + &tn[2..4] + ".2";

}

macro_rules! from_be_bytes {
    ($t:ty, $v:expr) => {
        {
            let mut bytes = [0u8; size_of::<$t>()];
            bytes.clone_from_slice($v);

            <$t>::from_be_bytes(bytes)
        }
    }
}

pub struct DriverStation {
    team_number: u16,
    rio_socket: UdpSocket,
    mode: Mode,
    enabled: bool,
    sequence: u16,
    e_stopped: bool,
    fms_connected: bool,
    alliance: Alliance,
    restart_rio: bool,
    restart_code: bool,
}

impl DriverStation {
    pub async fn new(team: u16) -> Self {
        return Self {
            team_number: team,
            rio_socket: UdpSocket::bind("172.0.0.1:4000").expect("Error: Failed to bind socket"),
            mode: Mode::Teleop,
            enabled: false,
            sequence: 0,
            e_stopped: false,
            fms_connected: false,
            alliance: Alliance::Red { val: 1 },
            restart_rio: false,
            restart_code: false,
        }
    }

    pub async fn receive_rio_udp(self) -> FromRioUdpPacket {
        let mut buf = [0u8; 1500];
        let (num_bytes, _) = self.rio_socket.recv_from(&mut buf).unwrap();
        
        return FromRioUdpPacket::from_packet(&buf[..num_bytes]);
    }

    pub async fn send_rio_udp(self) -> Result<usize, std::io::Error> {
        let mut control: u8 = 0;

        control += if self.e_stopped { 0b10000000 } else { 0 }; 
        control += if self.fms_connected { 0b1000 } else { 0 };
        control += if self.enabled { 0b100 } else { 0 };
        control += match self.mode {
            Mode::Teleop => 0,
            Mode::Test => 1,
            Mode::Autonomous => 2,
            _ => 3,
        };

        let mut request: u8 = 0;

        request += if self.restart_rio { 0b1000 } else { 0 };
        request += if self.restart_code { 0b100 } else { 0 };

        let main_buf: Vec<u8> = vec![
            u16::to_be_bytes(self.sequence)[0],
            u16::to_be_bytes(self.sequence)[1],
            0x01,
            control,
            request,
            alliance_to_int(self.alliance),
        ];

        return self.rio_socket.send_to(&main_buf[..], "10.82.30.2:1510");
    }
}

pub struct FromRioUdpPacket {
    sequence: u16,
    e_stopped: bool,
    brownout: bool,
    code_start: bool,
    enabled: bool,
    mode: Mode,
    robot_code: bool,
    is_rio: bool,
    test: bool,
    autonomous: bool,
    teleop_code: bool,
    disabled: bool,
    battery: f32,
    request_date: bool,
    tags: Vec<FromRioTag>,
}

impl FromRioUdpPacket {
    pub fn from_packet(buf: &[u8]) -> Self {
        let mut result =  Self {
            sequence: u16::from_be_bytes(buf[..2].try_into().unwrap()),
            e_stopped: buf[3] & 0b10000000 != 0,
            brownout: buf[3] & 0b10000 != 0,
            code_start: buf[3] & 0b1000 != 0,
            enabled: buf[3] & 0b100 != 0,
            mode: match buf[3] % 0b100 {
                0 => Mode::Teleop,
                1 => Mode::Test,
                2 => Mode::Autonomous,
                _ => Mode::Invalid,
            },
            robot_code: buf[4] & 0b100000 != 0,
            is_rio: buf[4] & 0b10000 != 0,
            test: buf[4] & 0b1000 != 0,
            autonomous: buf[4] & 0b100 != 0,
            teleop_code: buf[4] & 0b10 != 0,
            disabled: buf[4] % 0b10 != 0,
            battery: buf[5] as f32 + ( buf[6] as f32 / 256f32 ), 
            request_date: buf[7] != 0,
            tags: Vec::new(),
        };

        let mut tags: Vec<u8> = buf[7..].to_vec();

        while !tags.is_empty() {
            let mut tag = FromRioTag {
                size: tags[0],
                joystick: None,
                disk: None,
                cpu: None,
                ram: None,
                pdp: None,
                unknown: None,
                can: None,
            };

             match tags[1] {
                 0x01 => tag.joystick = Some(JoystickOutput {
                    joysticks: from_be_bytes!(u32, &tags[2..6]),
                    left_rumble: from_be_bytes!(u16, &tags[6..8]),
                    right_rumble: from_be_bytes!(u16, &tags[8..10]),
                 }),
                 0x04 => tag.disk = Some(DiskInfo {
                    bytes_available: from_be_bytes!(u32, &tags[2..6]),
                 }),
                 0x05 => tag.cpu = Some(CpuInfo {
                     cpu_count: from_be_bytes!(f32, &tags[2..6]),
                     cpu_time_critical_per: from_be_bytes!(f32, &tags[6..10]),
                     cpu_above_normal_per: from_be_bytes!(f32, &tags[10..14]),
                     cpu_normal_per: from_be_bytes!(f32, &tags[14..18]),
                     cpu_low_per: from_be_bytes!(f32, &tags[18..22]),
                 }),
                 0x06 => tag.ram = Some(RamInfo {
                     block: from_be_bytes!(u32, &tags[2..6]),
                     free_space: from_be_bytes!(u32, &tags[6..10]),
                 }),
                 0x08 => tag.pdp = Some(PdpLog {
                     unknown: tags[2],
                     pdp_stats_1: from_be_bytes!(u64, &tags[3..11]),
                     pdp_stats_2: from_be_bytes!(u64, &tags[11..19]),
                     pdp_stats_3: from_be_bytes!(u32, &tags[19..23]),
                     pdp_stats_4: tags[23],
                     unknown_2: from_be_bytes!(u16, &tags[24..26]),
                     unknown_3: tags[26],
                 }),
                 0x09 => tag.unknown = Some(UnknownTag {
                    sec_1: from_be_bytes!(u64, &tags[2..10]),
                    sec_2: tags[10],
                 }),
                 0x0e => tag.can = Some(CanMetrics {
                     utilization: from_be_bytes!(f32, &tags[2..6]),
                     bus_off: from_be_bytes!(u32, &tags[6..10]),
                     tx_full: from_be_bytes!(u32, &tags[10..14]),
                     rx_errors: tags[14],
                     tx_errors: tags[15],
                 }),
                _ => ()
            };

            for i in 0..tag.size {
                tags.remove(0);
            }

            result.tags.append(&mut vec![tag]);
        }



        return result;
    }
}


pub struct FromRioTag {
    size: u8,
    joystick: Option<JoystickOutput>,
    disk: Option<DiskInfo>,
    cpu: Option<CpuInfo>,
    ram: Option<RamInfo>,
    pdp: Option<PdpLog>,
    unknown: Option<UnknownTag>,
    can: Option<CanMetrics>
        ,
}

pub struct JoystickOutput {
    joysticks: u32,
    left_rumble: u16,
    right_rumble: u16,
}

pub struct DiskInfo {
    bytes_available: u32,
}

pub struct CpuInfo {
    cpu_count: f32, // 0x02 on RoboRIO
    cpu_time_critical_per: f32,
    cpu_above_normal_per: f32,
    cpu_normal_per: f32,
    cpu_low_per: f32,
}


pub struct RamInfo {
    block: u32,
    free_space: u32,
}

// TODO: Figure ut how this behaves with REV PDH
pub struct PdpLog {
    unknown: u8,
    pdp_stats_1: u64,
    pdp_stats_2: u64,
    pdp_stats_3: u32,
    pdp_stats_4: u8,
    unknown_2: u16,
    unknown_3: u8,
}

pub struct UnknownTag {
    sec_1: u64,
    sec_2: u8,
}

pub struct CanMetrics {
    utilization: f32,
    bus_off: u32,
    tx_full: u32,
    rx_errors: u8,
    tx_errors: u8,
}

#[repr(u8)]
pub enum Mode {
    Teleop = 0,
    Test = 1,
    Autonomous = 2,
    Invalid = 3,
}

pub enum Alliance {
    Red{ val: u8 },
    Blue{ val: u8 },
}

fn alliance_to_int(alliance: Alliance) -> u8 {
    return match alliance {
        Alliance::Red { val } => (val - 1) % 3,
        Alliance::Blue { val } => ((val - 1) % 3) + 3,
    }
}

fn alliance_from_int(num: u8) -> Alliance {
    return if num < 3 {
        Alliance::Red { val: num % 3 + 1 }
    } else {
        Alliance::Blue { val:  num % 3 + 1 }
    }
}

