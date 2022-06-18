use probe_rs::architecture::arm::Pins;
use probe_rs::{Permissions, Probe};
use std::thread;
use std::time::Duration;

fn main() {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Trace,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    let probes = Probe::list_all();
    let probe = probes[0].open().unwrap();
    // This path doesn't work.
    // let interface = probe.try_as_dap_probe().unwrap();

    // This path works, presumably because connecting
    // causes the pins to switch to outputs.
    let mut session = probe
        .attach("ATSAML10E16A", Permissions::default())
        .unwrap();
    let interface = session.get_arm_interface().unwrap();

    let mut pin_out = Pins(0);
    let mut pin_mask = Pins(0);

    pin_mask.set_nreset(true);
    pin_mask.set_swclk_tck(true);
    pin_mask.set_swdio_tms(true);

    pin_out.set_nreset(true);
    pin_out.set_swclk_tck(true);
    pin_out.set_swdio_tms(true);
    let val = interface
        .swj_pins(pin_out.0 as u32, pin_mask.0 as u32, 0)
        .unwrap();
    println!("Value: {}", val);
    thread::sleep(Duration::from_millis(100));

    pin_out.set_nreset(false);
    pin_out.set_swclk_tck(false);
    pin_out.set_swdio_tms(false);
    let val = interface
        .swj_pins(pin_out.0 as u32, pin_mask.0 as u32, 0)
        .unwrap();
    println!("Value: {}", val);
    thread::sleep(Duration::from_millis(100));

    pin_out.set_nreset(true);
    pin_out.set_swclk_tck(true);
    pin_out.set_swdio_tms(true);
    let val = interface
        .swj_pins(pin_out.0 as u32, pin_mask.0 as u32, 0)
        .unwrap();
    println!("Value: {}", val);

    println!("Hello, world!");
}
