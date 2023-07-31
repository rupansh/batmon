use tokio_udev::Device;

use super::BatLvl;

/// extract battery capacity
pub fn extract_battery_cap(ev: &Device) -> BatLvl {
    ev.property_value("POWER_SUPPLY_CAPACITY")
        .expect("battery does not advertise capacity?!")
        .to_str()
        .expect("battery capacity is not valid utf8?!")
        .parse::<BatLvl>()
        .expect("battery capacity is not a valid u8?!")
}
