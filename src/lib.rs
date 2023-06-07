pub mod driverstation;

#[cfg(test)]
mod tests {
    #[test]
    fn one_digit_team_number() {
        let result = crate::driverstation::team_number_to_ip(8);
        assert_eq!(result, "10.00.08.2")
    }

    #[test]
    fn two_digit_team_number() {
        let result = crate::driverstation::team_number_to_ip(82);
        assert_eq!(result, "10.00.82.2")
    }

    #[test]
    fn three_digit_team_number() {
        let result = crate::driverstation::team_number_to_ip(823);
        assert_eq!(result, "10.08.23.2")
    }

    #[test]
    fn four_digit_team_number() {
    let result = crate::driverstation::team_number_to_ip(8230);
    assert_eq!(result, "10.82.30.2")
    }
}