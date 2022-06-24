use anyhow::{Context, Result};
use probe_rs::architecture::arm::ArmProbeInterface;
use probe_rs::architecture::arm::Pins;
use probe_rs::{Permissions, Probe};
use std::thread;
use std::time::Duration;

/// Performs a cold plugging sequence.
fn do_cold_plug(interface: &mut Box<dyn ArmProbeInterface>) -> Result<()> {
    let mut pin_out = Pins(0);
    let mut pin_mask = Pins(0);

    log::warn!("atsaml10 do_cold_plug()");

    // 1 ms with reset high.
    pin_out.set_nreset(true);
    pin_mask.set_nreset(true);
    interface.swj_pins(pin_out.0 as u32, pin_mask.0 as u32, 0)?;
    thread::sleep(Duration::from_millis(1));

    // 1 ms with reset low.
    pin_out.set_nreset(false);
    interface.swj_pins(pin_out.0 as u32, pin_mask.0 as u32, 0)?;
    thread::sleep(Duration::from_millis(1));

    // 1 ms with reset and clock low.
    pin_mask.set_swclk_tck(true);
    interface.swj_pins(pin_out.0 as u32, pin_mask.0 as u32, 0)?;
    thread::sleep(Duration::from_millis(1));

    // 1 ms with reset high.
    pin_mask.set_swclk_tck(false);
    pin_out.set_nreset(true);
    interface.swj_pins(pin_out.0 as u32, pin_mask.0 as u32, 0)?;
    thread::sleep(Duration::from_millis(1));

    Ok(())
}

fn main() -> Result<()> {
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

    do_cold_plug(interface).context("Failed to do cold plug")?;

    Ok(())
}
