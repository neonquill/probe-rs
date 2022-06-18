//! Sequences for the SAML10.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::ArmDebugSequence;
use crate::architecture::arm::ap::MemoryAp;
use crate::architecture::arm::{communication_interface::DapProbe, ArmProbeInterface, Pins};

/// The sequence handle for the ATSAML10 set of chips.
pub struct Atsaml10(());

impl Atsaml10 {
    const DSU_ADDR: u32 = 0x41002100;
    const DSU_STATUSB_ADDR: u32 = Self::DSU_ADDR + 0x2;
    const DSU_DID_ADDR: u32 = Self::DSU_ADDR + 0x18;

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

        Ok(true)
    }
}

impl ArmDebugSequence for Atsaml10 {
    fn reset_hardware_assert(&self, interface: &mut dyn DapProbe) -> Result<(), crate::Error> {
        log::warn!("atsaml10 reset_hardware_assert");

        let mut pin_out = Pins(0);
        let mut pin_mask = Pins(0);

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

        // Sleep for 10 ms more.
        thread::sleep(Duration::from_millis(10));

        Ok(())
    }

    fn debug_device_unlock(
        &self,
        interface: &mut Box<dyn ArmProbeInterface>,
        default_ap: MemoryAp,
        permissions: &crate::Permissions,
    ) -> Result<(), crate::Error> {
        let mut memory = interface.memory_interface(default_ap)?;
        log::warn!("XXX");
        self.is_core_unlocked(&mut memory)?;
        Ok(())
    }
}
