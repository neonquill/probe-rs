//! Sequences for the SAML10.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::ArmDebugSequence;
use crate::architecture::arm::ap::MemoryAp;
use crate::architecture::arm::{communication_interface::DapProbe, ArmProbeInterface, Pins};

/// An error occurred when executing an ARM debug sequence
#[derive(thiserror::Error, Debug)]
pub enum AtsamDebugSequenceError {
    /// Tried to enter reset phase extension, but it failed.
    #[error("Tried to enter reset phase extension but DSU.STATUSA.CRSTEXT was not set")]
    CpuResetExtensionFailed,

    /// Tried to enter Boot ROM interactive mode, but it didn't work.
    #[error("Failed to enter Boot ROM interactive mode.")]
    BootRomInteractiveModeFailed,

    /// We tried to issue the Chip Erase command, but the device said it was invalid.
    #[error("Chip Erase failed due to invalid command")]
    ChipEraseInvalidCommand,
}

/// The sequence handle for the ATSAML10 set of chips.
pub struct Atsaml10(());

impl Atsaml10 {
    const DSU_ADDR: u32 = 0x41002100;
    const DSU_STATUSA_ADDR: u32 = Self::DSU_ADDR + 0x1;
    const DSU_STATUSB_ADDR: u32 = Self::DSU_ADDR + 0x2;
    const DSU_DID_ADDR: u32 = Self::DSU_ADDR + 0x18;
    const DSU_BCC0_ADDR: u32 = Self::DSU_ADDR + 0x20;
    const DSU_BCC1_ADDR: u32 = Self::DSU_ADDR + 0x24;
    // XXX Handle registers better.
    const CRSTEXT_BIT: u8 = 1 << 1;
    const BCCD1_BIT: u8 = 1 << 7;
    // Boot Interactive Mode commands (14.4.5.9).
    // Enter Interactive Mode.
    const CMD_INIT: u32 = 0x444247_55;
    // Exit Interactive Mode.
    const CMD_EXIT: u32 = 0x444247_AA;
    // ChipErease for SAM L10.
    const CMD_CHIPERASE: u32 = 0x444247_E3;
    // Boot Interactive Mode Status (14.4.5.10).
    // Debugger start communication.
    const SIG_COMM: u32 = 0xEC0000_20;
    // Dubber command success.
    const SIG_CMD_SUCCESS: u32 = 0xEC0000_21;
    // Valid command.
    const SIG_CMD_VALID: u32 = 0xEC0000_24;
    // Boot ROM ok to exit.
    const SIG_BOOTOK: u32 = 0xEC0000_39;

    /// Create a new sequence handle for the atsaml10.
    pub fn create() -> Arc<dyn ArmDebugSequence> {
        Arc::new(Self(()))
    }

    /// Returns true when the core is unlocked and false when it is locked.
    fn is_core_unlocked(&self, interface: &mut crate::Memory) -> Result<bool, crate::Error> {
        let did = interface.read_word_32((Self::DSU_DID_ADDR).into())?;
        log::warn!("Read DID {:x}", did);

        let statusb = interface.read_word_8((Self::DSU_STATUSB_ADDR).into())?;
        log::warn!("Read STATUSB {:x}", statusb);

        let dal = statusb & 0b11;
        log::warn!("DAL {}", dal);

        Ok(dal == 2)
    }

    /// Performs a cold plugging sequence.
    fn do_cold_plug(&self, interface: &mut dyn DapProbe) -> Result<(), crate::Error> {
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
}

impl ArmDebugSequence for Atsaml10 {
    fn reset_hardware_assert(&self, interface: &mut dyn DapProbe) -> Result<(), crate::Error> {
        log::warn!("atsaml10 reset_hardware_assert");

        self.do_cold_plug(interface)?;

        // XXX Check to make sure this succeeded?
        // XXX Check CRSTEXT in DSU.STATUSA

        Ok(())
    }

    fn debug_device_unlock(
        &self,
        interface: &mut Box<dyn ArmProbeInterface>,
        default_ap: MemoryAp,
        permissions: &crate::Permissions,
    ) -> Result<(), crate::Error> {
        {
            let mut memory = interface.memory_interface(default_ap)?;
            log::warn!("XXX");
            let unlocked = self.is_core_unlocked(&mut memory)?;

            // See if we're allowed to erase the chip.
            if let Err(_e) = permissions.erase_all() {
                if !unlocked {
                    // XXX Right way to do this?
                    log::warn!("Chip is locked (debug access level: DAL == 0)");
                    log::warn!("Need to allow chip erase to continue.");
                    log::warn!("Going to erase anyway!!!!");
                    // XXX ignore for now.
                    // return Err(e);
                } else {
                    // User doesn't want us to erase, just continue.
                    return Ok(());
                }
            }

            log::warn!("Erasing chip");

            // At this point we're allowed to erase.
            // Do it even if we don't need to.
            // XXX Is this what I want to do?

            // Make sure the CRSTEXT bit is set to indicate we're in the
            // reset extension phase.
            let statusa = memory.read_word_8((Self::DSU_STATUSA_ADDR).into())?;
            if (statusa & Self::CRSTEXT_BIT) == 0 {
                // XXX Better warning message?
                log::warn!("Reset extension failed, need `--connect-under-reset`?");
                return Err(crate::Error::architecture_specific(
                    AtsamDebugSequenceError::CpuResetExtensionFailed,
                ));
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

            // Request Boot ROM Interactive mode entry (14.4.5.1.1).
            memory.write_word_32((Self::DSU_BCC0_ADDR).into(), Self::CMD_INIT)?;

            log::warn!("XXXa5");

            // Check for SIG_COMM status in DSU.BCC1.
            let status = memory.read_word_32((Self::DSU_BCC1_ADDR).into())?;
            // Possibly I need to wait for the bit to be set?
            if status != Self::SIG_COMM {
                log::warn!("XXX status wrong: {:x}", status);
                return Err(crate::Error::architecture_specific(
                    AtsamDebugSequenceError::BootRomInteractiveModeFailed,
                ));
            }

            log::warn!("XXXa6");

            // Issue the Chip Erase command (14.4.5.4.1).
            memory.write_word_32((Self::DSU_BCC0_ADDR).into(), Self::CMD_CHIPERASE)?;

            // Check to see if the command was valid.
            let status = memory.read_word_32((Self::DSU_BCC1_ADDR).into())?;
            if status != Self::SIG_CMD_VALID {
                log::warn!("XXX status wrong: {:x}", status);
                return Err(crate::Error::architecture_specific(
                    AtsamDebugSequenceError::ChipEraseInvalidCommand,
                ));
            }

            log::warn!("XXXa7");

            // Poll for status update.
            let mut status = 0;
            for i in 0..20 {
                status = memory.read_word_32((Self::DSU_BCC1_ADDR).into())?;
                if status != Self::SIG_CMD_VALID && status != 0 {
                    // XXX Change this to trace.
                    log::warn!("Received status update after {} cycles", i);
                    break;
                }
                // No status update, wait for a while before trying again.
                thread::sleep(Duration::from_secs(1));
            }

            log::warn!("XXXa8");

            // Make sure we were successful.
            if status != Self::SIG_CMD_SUCCESS {
                // XXX is warn the right message?
                log::warn!("XXX Chip Erase failed!");
                // XXX reset to park?
            } else {
                // XXX warn?
                log::warn!("XXX Chip Erase succeeded");
            }

            log::warn!("XXXa9");
        }
        // Now exit to park mode (14-9).

        // XXX Duplicated from above.

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

        // XXX End duplicated from above.

        let mut memory = interface.memory_interface(default_ap)?;

        // XXX Duplicated from above.

        // Make sure the CRSTEXT bit is set to indicate we're in the
        // reset extension phase.
        let statusa = memory.read_word_8((Self::DSU_STATUSA_ADDR).into())?;
        if (statusa & Self::CRSTEXT_BIT) == 0 {
            // XXX Better warning message?
            log::warn!("Reset extension failed, need `--connect-under-reset`?");
            return Err(crate::Error::architecture_specific(
                AtsamDebugSequenceError::CpuResetExtensionFailed,
            ));
        }

        log::warn!("XXXa9a");

        // Clear the CRSTEXT bit.
        memory.write_word_8((Self::DSU_STATUSA_ADDR).into(), Self::CRSTEXT_BIT)?;
        log::warn!("XXXa9b");

        // Wait 5ms for CPU to execute Boot ROM failure analysis and security
        // checks.
        thread::sleep(Duration::from_millis(5));

        log::warn!("XXXa9c");

        // Check to see if there were any errors.
        let statusb = memory.read_word_8((Self::DSU_STATUSB_ADDR).into())?;
        if (statusb & Self::BCCD1_BIT) != 0 {
            log::warn!("Boot discovered errors, continuing: XXX");
            // XXX Go read the error code and show to the user.
        }

        log::warn!("XXXa9d");

        // XXX Not sure I'm doing this right.
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

        log::warn!("Chip unlock succeeded!");

        Ok(())
    }
}
