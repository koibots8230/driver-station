use std::borrow::ToOwned;
use std::net::UdpSocket;

// NOTE: Produces ip addresses with leading zeros. This should NOT be a problem according to the rust docs. https://doc.rust-lang.org/std/net/struct.Ipv4Addr.html#textual-representation
pub fn team_number_to_ip(team_number: u16) -> String {
    let mut tn = team_number.to_string();
    tn = "0".repeat(4 - tn.len()) + &tn;
    return "10.".to_owned() + &tn[0..2] + "." + &tn[2..4] + ".2";
}

pub struct DriverStation {
    team_number: u16,
    udp_socket: UdpSocket,
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
            udp_socket: UdpSocket::bind("172.0.0.1:4000").expect("Error: Failed to bind socket"),
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

    pub async fn from_rio_udp(self) -> FromRioUdpPacket {
        let mut buf = [0u8; 1500];
        let (num_bytes, _) = self.udp_socket.recv_from(&mut buf).unwrap();
        
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

        return self.udp_socket.send_to(&main_buf[..], "10.82.30.2:1510");
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
    auton: bool,
    teleop_code: bool,
    disabled: bool,
    battery: f32,
    request_date: bool,
    tags: Option<Vec<FromRioTag>>,
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
            auton: buf[4] & 0b100 != 0,
            teleop_code: buf[4] & 0b10 != 0,
            disabled: buf[4] % 0b10 != 0,
            battery: buf[5] as f32 + ( buf[6] as f32 / 256f32 ), 
            request_date: buf[7] != 0,
            tags: None,
        };

        let mut size = buf.len();
        


        return result;
    }
}


pub struct FromRioTag {
    size: u8,
    joystick: Option<JoystickOutput>,
    disk: Option<DiskInfo>,
    ram: Option<RamInfo>,
    pdp: Option<PdpLog>,
    unkown: Option<UnkownTag>,
    can: Option<CanMetrics>
        ,
}

pub struct JoystickOutput {
    joysticks: u64,
    left_rumble: u16,
    right_rumble: u16,
}

pub struct DiskInfo {
    cpu_count: f32, // 0x02 on RoboRIO
    cpu_time_critacal_per: f32,
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
    unkown: u8,
    pdp_stats_1: u64,
    pdp_stats_2: u64,
    pdp_stats_3: u32,
    pdp_stats_4: u8,
    unkown_2: u16,
    unkown_3: u8,
}

pub struct UnkownTag {
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
    match alliance {
        Alliance::Red { val } => return ( val - 1 ) % 3,
        Alliance::Blue { val } => return ( ( val - 1 ) % 3 ) + 3,
    }
}

fn alliance_from_int(num: u8) -> Alliance {
    return if num < 3 {
        Alliance::Red { val: num % 3 + 1 }
    } else {
        Alliance::Blue { val:  num % 3 + 1 }
    }
    ;
}

