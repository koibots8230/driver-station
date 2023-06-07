use driver_station_rs::driverstation::DriverStation;
use driver_station_rs::driverstation::state::DriverStationState;

#[tokio::main]
async fn main() {
    let mut ds = DriverStation::new(8230);
    let mut state: DriverStationState = DriverStationState::default();

    ds.init(&mut state.to_owned());

    let i = 65536;
    while i > 0 {
        println!("{:?}", state);
    }

    ds.quit();
}
