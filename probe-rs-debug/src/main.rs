use anyhow::{anyhow, Context, Result};
use probe_rs::architecture::arm::ap::MemoryAp;
use probe_rs::architecture::arm::{ApAddress, ArmProbeInterface, DpAddress, Pins};
use probe_rs::Memory;
use probe_rs::{Permissions, Probe};
use std::thread;
use std::time::Duration;

pub struct Atsaml10(());

impl Atsaml10 {
    const DSU_ADDR: u32 = 0x41002100;
    const DSU_STATUSA_ADDR: u32 = Self::DSU_ADDR + 0x1;
    const DSU_STATUSB_ADDR: u32 = Self::DSU_ADDR + 0x2;
    const _DSU_DID_ADDR: u32 = Self::DSU_ADDR + 0x18;
    const DSU_BCC0_ADDR: u32 = Self::DSU_ADDR + 0x20;
    const DSU_BCC1_ADDR: u32 = Self::DSU_ADDR + 0x24;
    // XXX Handle registers better.
    const CRSTEXT_BIT: u8 = 1 << 1;
    const BCCD1_BIT: u8 = 1 << 7;
    // Boot Interactive Mode commands (14.4.5.9).
    // Enter Interactive Mode.
    const _CMD_INIT: u32 = 0x444247_55;
    // Exit Interactive Mode.
    const CMD_EXIT: u32 = 0x444247_AA;
    // ChipErease for SAM L10.
    const _CMD_CHIPERASE: u32 = 0x444247_E3;
    // Boot Interactive Mode Status (14.4.5.10).
    // Debugger start communication.
    const _SIG_COMM: u32 = 0xEC0000_20;
    // Dubber command success.
    const _SIG_CMD_SUCCESS: u32 = 0xEC0000_21;
    // Valid command.
    const _SIG_CMD_VALID: u32 = 0xEC0000_24;
    // Boot ROM ok to exit.
    const SIG_BOOTOK: u32 = 0xEC0000_39;

    /// Performs a cold plugging sequence.
    fn do_cold_plug(&self, interface: &mut Box<dyn ArmProbeInterface>) -> Result<()> {
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

    fn exit_reset_extension(&self, memory: &mut Memory) -> Result<()> {
        // Make sure the CRSTEXT bit is set to indicate we're in the
        // reset extension phase.
        let statusa = memory.read_word_8((Self::DSU_STATUSA_ADDR).into())?;
        if (statusa & Self::CRSTEXT_BIT) == 0 {
            // XXX Better warning message?
            log::warn!("Reset extension failed, need `--connect-under-reset`?");
            return Err(anyhow!("CPU Reset Extension failed"));
        }

        log::warn!("XXXa1");

        // Clear the CRSTEXT bit.
        memory.write_word_8((Self::DSU_STATUSA_ADDR).into(), Self::CRSTEXT_BIT)?;

        log::warn!("XXXa2");

        // Wait 5ms for CPU to execute Boot ROM failure analysis and security
        // checks.
        thread::sleep(Duration::from_millis(5));

        log::warn!("XXXa3");

        // Check to see if there were any errors.
        let statusb = memory.read_word_8((Self::DSU_STATUSB_ADDR).into())?;
        if (statusb & Self::BCCD1_BIT) != 0 {
            log::warn!("Boot discovered errors, continuing: XXX");
            // XXX Go read the error code and show to the user.
        }

        log::warn!("XXXa4");

        Ok(())
    }

    fn exit_interactive_mode(&self, memory: &mut Memory) -> Result<()> {
        memory.write_word_32((Self::DSU_BCC0_ADDR).into(), Self::CMD_EXIT)?;

        log::warn!("XXXaa");

        // Poll for status update.
        for _ in 0..20 {
            let statusb = memory.read_word_8((Self::DSU_STATUSB_ADDR).into())?;
            if (statusb & Self::BCCD1_BIT) != 0 {
                let status = memory.read_word_32((Self::DSU_BCC1_ADDR).into())?;
                if status != Self::SIG_BOOTOK {
                    log::warn!("Failed to exit to park!: status {:x}", status);
                    // XXX Error!
                }
            }
            // No status update, wait for a while before trying again.
            thread::sleep(Duration::from_millis(50));
        }

        Ok(())
    }

    fn get_dal(&self, interface: &mut crate::Memory) -> Result<u8> {
        let statusb = interface.read_word_8((Self::DSU_STATUSB_ADDR).into())?;
        log::warn!("Read STATUSB {:x}", statusb);

        let dal = statusb & 0b11;
        log::warn!("DAL {}", dal);

        Ok(dal)
    }
}

fn main() -> Result<()> {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Trace,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )?;

    let probes = Probe::list_all();
    let mut probe = probes[0].open()?;
    // This path doesn't work.
    // let interface = probe.try_as_dap_probe()?;

    // This path works, presumably because connecting
    // causes the pins to switch to outputs.
    // But now I don't want to attach and do everything automatically.
    // log::warn!("MANUAL probe.attach");
    //let mut session = probe.attach("ATSAML10E16A", Permissions::default())?;

    // Attach without running any init routines (?).
    log::warn!("MANUAL attach_to_unspecified");
    probe.attach_to_unspecified()?;

    log::warn!("MANUAL try_into_arm_interface");
    let interface = probe.try_into_arm_interface().map_err(|(_, e)| e)?;

    log::warn!("MANUAL initialize");
    let mut interface = interface.initialize_unspecified()?;

    let atsaml10 = Atsaml10(());

    // First, do a cold plug sequence.
    // XXX Something before this is sending commands...
    log::warn!("MANUAL cold plug");
    atsaml10
        .do_cold_plug(&mut interface)
        .context("Failed to do cold plug")?;

    log::warn!("MANUAL port");
    let port = ApAddress {
        dp: DpAddress::Default,
        ap: 0,
    };

    log::warn!("MANUAL read_raw_ap_register");
    let val = interface.read_raw_ap_register(port, 0)?;
    log::warn!("val {}", val);

    /*
    log::warn!("MANUAL memory_interface");
    let mut memory = interface.memory_interface(default_memory_ap)?;

    // Now follow the CMD_EXIT to Park mode diagram (14-9 page 81).
    atsaml10.exit_reset_extension(&mut memory)?;
    // By not clearing BREXT here we go into park mode.
    atsaml10.exit_interactive_mode(&mut memory)?;

    // Make sure the debug access level is 2 (unlocked).
    let dal = atsaml10.get_dal(&mut memory)?;
    if dal != 2 {
        return Err(anyhow!(
            "Device is locked. DAL == {} != 2. Try erasing the chip to clear.",
            dal
        ));
    }
     */

    Ok(())
}
