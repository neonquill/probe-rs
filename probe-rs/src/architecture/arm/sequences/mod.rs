//! Debug sequences to operate special requirements ARM targets.

pub mod atsam;
pub mod nrf53;
pub mod nxp;
pub mod stm32;

use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use probe_rs_target::CoreType;

use crate::architecture::arm::core::armv7a_debug_regs::Armv7DebugRegister;
use crate::{
    architecture::arm::{ArmProbeInterface, DapError},
    core::MemoryMappedRegister,
    DebugProbeError, Memory,
};

use super::{
    ap::{AccessPortError, MemoryAp},
    communication_interface::{DapProbe, Initialized},
    dp::{Abort, Ctrl, DpAccess, Select, DPIDR},
    ArmCommunicationInterface, DpAddress, Pins, PortType, Register,
};

/// An error occurred when executing an ARM debug sequence
#[derive(thiserror::Error, Debug)]
pub enum ArmDebugSequenceError {
    /// Debug base address is required but not specified
    #[error("Core access requries debug_base to be specified, but it is not")]
    DebugBaseNotSpecified,

    /// CTI base address is required but not specified
    #[error("Core access requries cti_base to be specified, but it is not")]
    CtiBaseNotSpecified,
}

/// The default sequences that is used for ARM chips that do not specify a specific sequence.
pub struct DefaultArmSequence(pub(crate) ());

impl DefaultArmSequence {
    /// Creates a new default ARM debug sequence.
    pub fn create() -> Arc<dyn ArmDebugSequence> {
        Arc::new(Self(()))
    }
}

impl ArmDebugSequence for DefaultArmSequence {}

/// ResetCatchSet for Cortex-A devices
fn armv7a_reset_catch_set(core: &mut Memory, debug_base: Option<u64>) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv7a_debug_regs::Dbgprcr;

    let debug_base = debug_base.ok_or_else(|| {
        crate::Error::architecture_specific(ArmDebugSequenceError::DebugBaseNotSpecified)
    })?;

    let address = Dbgprcr::get_mmio_address(debug_base);
    let mut dbgprcr = Dbgprcr(core.read_word_32(address)?);

    dbgprcr.set_hcwr(true);

    core.write_word_32(address, dbgprcr.into())?;

    Ok(())
}

/// ResetCatchClear for Cortex-A devices
fn armv7a_reset_catch_clear(
    core: &mut Memory,
    debug_base: Option<u64>,
) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv7a_debug_regs::Dbgprcr;

    let debug_base = debug_base.ok_or_else(|| {
        crate::Error::architecture_specific(ArmDebugSequenceError::DebugBaseNotSpecified)
    })?;

    let address = Dbgprcr::get_mmio_address(debug_base);
    let mut dbgprcr = Dbgprcr(core.read_word_32(address)?);

    dbgprcr.set_hcwr(false);

    core.write_word_32(address, dbgprcr.into())?;

    Ok(())
}

fn armv7a_reset_system(
    interface: &mut Memory,
    debug_base: Option<u64>,
) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv7a_debug_regs::{Dbgprcr, Dbgprsr};

    let debug_base = debug_base.ok_or_else(|| {
        crate::Error::architecture_specific(ArmDebugSequenceError::DebugBaseNotSpecified)
    })?;

    // Request reset
    let address = Dbgprcr::get_mmio_address(debug_base);
    let mut dbgprcr = Dbgprcr(interface.read_word_32(address)?);

    dbgprcr.set_cwrr(true);

    interface.write_word_32(address, dbgprcr.into())?;

    // Wait until reset happens
    let address = Dbgprsr::get_mmio_address(debug_base);

    loop {
        let dbgprsr = Dbgprsr(interface.read_word_32(address)?);
        if dbgprsr.sr() {
            break;
        }
    }

    Ok(())
}

/// DebugCoreStart for v7 Cortex-A devices
fn armv7a_core_start(core: &mut Memory, debug_base: Option<u64>) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv7a_debug_regs::{Dbgdsccr, Dbgdscr, Dbgdsmcr, Dbglar};

    let debug_base = debug_base.ok_or_else(|| {
        crate::Error::architecture_specific(ArmDebugSequenceError::DebugBaseNotSpecified)
    })?;
    log::debug!(
        "Starting debug for ARMv7-A core with registers at {:#X}",
        debug_base
    );

    // Lock OS register access to prevent race conditions
    let address = Dbglar::get_mmio_address(debug_base);
    core.write_word_32(address, Dbglar(0).into())?;

    // Force write through / disable caching for debugger access
    let address = Dbgdsccr::get_mmio_address(debug_base);
    core.write_word_32(address, Dbgdsccr(0).into())?;

    // Disable TLB matching and updates for debugger operations
    let address = Dbgdsmcr::get_mmio_address(debug_base);
    core.write_word_32(address, Dbgdsmcr(0).into())?;

    // Enable halting
    let address = Dbgdscr::get_mmio_address(debug_base);
    let mut dbgdscr = Dbgdscr(core.read_word_32(address)?);

    if dbgdscr.hdbgen() {
        log::debug!("Core is already in debug mode, no need to enable it again");
        return Ok(());
    }

    dbgdscr.set_hdbgen(true);
    core.write_word_32(address, dbgdscr.into())?;

    Ok(())
}

/// ResetCatchSet for ARMv8-A devices
fn armv8a_reset_catch_set(core: &mut Memory, debug_base: Option<u64>) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv8a_debug_regs::{Armv8DebugRegister, Edecr};

    let debug_base = debug_base.ok_or_else(|| {
        crate::Error::architecture_specific(ArmDebugSequenceError::DebugBaseNotSpecified)
    })?;

    let address = Edecr::get_mmio_address(debug_base);
    let mut edecr = Edecr(core.read_word_32(address)?);

    edecr.set_rce(true);

    core.write_word_32(address, edecr.into())?;

    Ok(())
}

/// ResetCatchClear for ARMv8-a devices
fn armv8a_reset_catch_clear(
    core: &mut Memory,
    debug_base: Option<u64>,
) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv8a_debug_regs::{Armv8DebugRegister, Edecr};

    let debug_base = debug_base.ok_or_else(|| {
        crate::Error::architecture_specific(ArmDebugSequenceError::DebugBaseNotSpecified)
    })?;

    let address = Edecr::get_mmio_address(debug_base);
    let mut edecr = Edecr(core.read_word_32(address)?);

    edecr.set_rce(false);

    core.write_word_32(address, edecr.into())?;

    Ok(())
}

fn armv8a_reset_system(
    interface: &mut Memory,
    debug_base: Option<u64>,
) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv8a_debug_regs::{Armv8DebugRegister, Edprcr, Edprsr};

    let debug_base = debug_base.ok_or_else(|| {
        crate::Error::architecture_specific(ArmDebugSequenceError::DebugBaseNotSpecified)
    })?;

    // Request reset
    let address = Edprcr::get_mmio_address(debug_base);
    let mut edprcr = Edprcr(interface.read_word_32(address)?);

    edprcr.set_cwrr(true);

    interface.write_word_32(address, edprcr.into())?;

    // Wait until reset happens
    let address = Edprsr::get_mmio_address(debug_base);

    loop {
        let edprsr = Edprsr(interface.read_word_32(address)?);
        if edprsr.sr() {
            break;
        }
    }

    Ok(())
}

/// DebugCoreStart for v8 Cortex-A devices
fn armv8a_core_start(
    core: &mut Memory,
    debug_base: Option<u64>,
    cti_base: Option<u64>,
) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv8a_debug_regs::{
        Armv8DebugRegister, CtiControl, CtiGate, CtiOuten, Edlar, Edscr,
    };

    let debug_base = debug_base.ok_or_else(|| {
        crate::Error::architecture_specific(ArmDebugSequenceError::DebugBaseNotSpecified)
    })?;
    let cti_base = cti_base.ok_or_else(|| {
        crate::Error::architecture_specific(ArmDebugSequenceError::CtiBaseNotSpecified)
    })?;

    log::debug!(
        "Starting debug for ARMv8-A core with registers at {:#X}",
        debug_base
    );

    // Lock OS register access to prevent race conditions
    let address = Edlar::get_mmio_address(debug_base);
    core.write_word_32(address, Edlar(0).into())?;

    // Configure CTI
    let mut cticontrol = CtiControl(0);
    cticontrol.set_glben(true);

    let address = CtiControl::get_mmio_address(cti_base);
    core.write_word_32(address, cticontrol.into())?;

    // Gate all events by default
    let address = CtiGate::get_mmio_address(cti_base);
    core.write_word_32(address, 0)?;

    // Configure output channels for halt and resume
    // Channel 0 - halt requests
    let mut ctiouten = CtiOuten(0);
    ctiouten.set_outen(0, 1);

    let address = CtiOuten::get_mmio_address(cti_base);
    core.write_word_32(address, ctiouten.into())?;

    // Channel 1 - resume requests
    let mut ctiouten = CtiOuten(0);
    ctiouten.set_outen(1, 1);

    let address = CtiOuten::get_mmio_address(cti_base) + 4;
    core.write_word_32(address, ctiouten.into())?;

    // Enable halting
    let address = Edscr::get_mmio_address(debug_base);
    let mut edscr = Edscr(core.read_word_32(address)?);

    if edscr.hde() {
        log::debug!("Core is already in debug mode, no need to enable it again");
        return Ok(());
    }

    edscr.set_hde(true);
    core.write_word_32(address, edscr.into())?;

    Ok(())
}

/// DebugCoreStart for Cortex-M devices
fn cortex_m_core_start(core: &mut Memory) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv7m::Dhcsr;

    let current_dhcsr = Dhcsr(core.read_word_32(Dhcsr::ADDRESS)?);

    // Note: Manual addition for debugging, not part of the original DebugCoreStart function
    if current_dhcsr.c_debugen() {
        log::debug!("Core is already in debug mode, no need to enable it again");
        return Ok(());
    }
    // -- End addition

    let mut dhcsr = Dhcsr(0);
    dhcsr.set_c_debugen(true);
    dhcsr.enable_write();

    core.write_word_32(Dhcsr::ADDRESS, dhcsr.into())?;

    Ok(())
}

/// ResetCatchClear for Cortex-M devices
fn cortex_m_reset_catch_clear(core: &mut Memory) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv7m::Demcr;

    // Clear reset catch bit
    let mut demcr = Demcr(core.read_word_32(Demcr::ADDRESS)?);
    demcr.set_vc_corereset(false);

    core.write_word_32(Demcr::ADDRESS, demcr.into())?;
    Ok(())
}

/// ResetCatchSet for Cortex-M devices
fn cortex_m_reset_catch_set(core: &mut Memory) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv7m::{Demcr, Dhcsr};

    // Request halt after reset
    let mut demcr = Demcr(core.read_word_32(Demcr::ADDRESS)?);
    demcr.set_vc_corereset(true);

    core.write_word_32(Demcr::ADDRESS, demcr.into())?;

    // Clear the status bits by reading from DHCSR
    let _ = core.read_word_32(Dhcsr::ADDRESS)?;

    Ok(())
}

/// ResetSystem for Cortex-M devices
fn cortex_m_reset_system(interface: &mut Memory) -> Result<(), crate::Error> {
    use crate::architecture::arm::core::armv7m::{Aircr, Dhcsr};

    let mut aircr = Aircr(0);
    aircr.vectkey();
    aircr.set_sysresetreq(true);

    interface.write_word_32(Aircr::ADDRESS, aircr.into())?;

    let start = Instant::now();

    while start.elapsed() < Duration::from_micros(50_0000) {
        let dhcsr = match interface.read_word_32(Dhcsr::ADDRESS) {
            Ok(val) => Dhcsr(val),
            Err(err) => {
                if let crate::Error::ArchitectureSpecific(ref arch_err) = err {
                    if let Some(AccessPortError::RegisterRead { .. }) =
                        arch_err.downcast_ref::<AccessPortError>()
                    {
                        // Some combinations of debug probe and target (in
                        // particular, hs-probe and ATSAMD21) result in
                        // register read errors while the target is
                        // resetting.
                        continue;
                    }
                }
                return Err(err);
            }
        };

        // Wait until the S_RESET_ST bit is cleared on a read
        if !dhcsr.s_reset_st() {
            return Ok(());
        }
    }

    Err(crate::Error::Probe(DebugProbeError::Timeout))
}

/// A interface to operate debug sequences for ARM targets.
///
/// Should be implemented on a custom handle for chips that require special sequence code.
pub trait ArmDebugSequence: Send + Sync {
    /// Assert a system-wide reset line nRST. This is based on the
    /// `ResetHardwareAssert` function from the [ARM SVD Debug Description].
    ///
    /// [ARM SVD Debug Description]: http://www.keil.com/pack/doc/cmsis/Pack/html/debug_description.html#resetHardwareAssert
    #[doc(alias = "ResetHardwareAssert")]
    fn reset_hardware_assert(&self, interface: &mut dyn DapProbe) -> Result<(), crate::Error> {
        log::warn!("reset_hardware_assert");
        let mut n_reset = Pins(0);
        n_reset.set_nreset(true);

        let _ = interface.swj_pins(0, n_reset.0 as u32, 0)?;

        Ok(())
    }

    /// De-Assert a system-wide reset line nRST. This is based on the
    /// `ResetHardwareDeassert` function from the [ARM SVD Debug Description].
    ///
    /// [ARM SVD Debug Description]: http://www.keil.com/pack/doc/cmsis/Pack/html/debug_description.html#resetHardwareDeassert
    #[doc(alias = "ResetHardwareDeassert")]
    fn reset_hardware_deassert(&self, memory: &mut Memory) -> Result<(), crate::Error> {
        log::warn!("reset_hardware_deassert");
        let interface = memory.get_arm_probe();

        let mut n_reset = Pins(0);
        n_reset.set_nreset(true);
        let n_reset = n_reset.0 as u32;

        let can_read_pins = interface.swj_pins(n_reset, n_reset, 0)? != 0xffff_ffff;

        if can_read_pins {
            let start = Instant::now();

            while start.elapsed() < Duration::from_secs(1) {
                if Pins(interface.swj_pins(n_reset, n_reset, 0)? as u8).nreset() {
                    return Ok(());
                }
            }

            Err(DebugProbeError::Timeout.into())
        } else {
            thread::sleep(Duration::from_millis(100));
            Ok(())
        }
    }

    /// Prepare the target debug port for connection. This is based on the
    /// `DebugPortSetup` function from the [ARM SVD Debug Description].
    ///
    /// [ARM SVD Debug Description]: http://www.keil.com/pack/doc/cmsis/Pack/html/debug_description.html#debugPortSetup
    #[doc(alias = "DebugPortSetup")]
    fn debug_port_setup(&self, interface: &mut Box<dyn DapProbe>) -> Result<(), crate::Error> {
        log::warn!("debug_port_setup");
        // TODO: Handle this differently for ST-Link?

        // TODO: Use atomic block

        // Ensure current debug interface is in reset state.
        interface.swj_sequence(51, 0x0007_FFFF_FFFF_FFFF)?;

        // Make sure the debug port is in the correct mode based on what the probe
        // has selected via active_protocol
        match interface.active_protocol() {
            Some(crate::WireProtocol::Jtag) => {
                // Execute SWJ-DP Switch Sequence SWD to JTAG (0xE73C).
                interface.swj_sequence(16, 0xE73C)?;
            }
            Some(crate::WireProtocol::Swd) => {
                // Execute SWJ-DP Switch Sequence JTAG to SWD (0xE79E).
                // Change if SWJ-DP uses deprecated switch code (0xEDB6).
                interface.swj_sequence(16, 0xE79E)?;
            }
            _ => {
                return Err(crate::Error::Probe(DebugProbeError::NotImplemented(
                    "Cannot detect current protocol",
                )));
            }
        }

        interface.swj_sequence(51, 0x0007_FFFF_FFFF_FFFF)?; // > 50 cycles SWDIO/TMS High.
        interface.swj_sequence(3, 0x00)?; // At least 2 idle cycles (SWDIO/TMS Low).

        // End of atomic block.

        // Read DPIDR to enable SWD interface.
        let _ = interface.raw_read_register(PortType::DebugPort, DPIDR::ADDRESS);

        // TODO: Figure a way how to do this.
        // interface.read_dpidr()?;

        Ok(())
    }

    /// Connect to the target debug port and power it up. This is based on the
    /// `DebugPortStart` function from the [ARM SVD Debug Description].
    ///
    /// [ARM SVD Debug Description]: http://www.keil.com/pack/doc/cmsis/Pack/html/debug_description.html#debugPortStart
    #[doc(alias = "DebugPortStart")]
    fn debug_port_start(
        &self,
        interface: &mut ArmCommunicationInterface<Initialized>,
        dp: DpAddress,
    ) -> Result<(), crate::DebugProbeError> {
        log::warn!("debug_port_start");
        // Clear all errors.
        // CMSIS says this is only necessary to do inside the `if powered_down`, but
        // without it here, nRF52840 faults in the next access.
        let mut abort = Abort(0);
        abort.set_orunerrclr(true);
        abort.set_wderrclr(true);
        abort.set_stkerrclr(true);
        abort.set_stkcmpclr(true);
        interface.write_dp_register(dp, abort)?;

        interface.write_dp_register(dp, Select(0))?;

        let ctrl = interface.read_dp_register::<Ctrl>(dp)?;

        let powered_down = !(ctrl.csyspwrupack() && ctrl.cdbgpwrupack());

        if powered_down {
            let mut ctrl = Ctrl(0);
            ctrl.set_cdbgpwrupreq(true);
            ctrl.set_csyspwrupreq(true);
            interface.write_dp_register(dp, ctrl)?;

            let start = Instant::now();
            let mut timeout = true;
            while start.elapsed() < Duration::from_micros(100_0000) {
                let ctrl = interface.read_dp_register::<Ctrl>(dp)?;
                if ctrl.csyspwrupack() && ctrl.cdbgpwrupack() {
                    timeout = false;
                    break;
                }
            }

            if timeout {
                return Err(DebugProbeError::Timeout);
            }

            // TODO: Handle JTAG Specific part

            // TODO: Only run the following code when the SWD protocol is used

            // Init AP Transfer Mode, Transaction Counter, and Lane Mask (Normal Transfer Mode, Include all Byte Lanes)
            let mut ctrl = Ctrl(0);
            ctrl.set_cdbgpwrupreq(true);
            ctrl.set_csyspwrupreq(true);
            ctrl.set_mask_lane(0b1111);
            interface.write_dp_register(dp, ctrl)?;

            let ctrl_reg: Ctrl = interface.read_dp_register(dp)?;
            if !(ctrl_reg.csyspwrupack() && ctrl_reg.cdbgpwrupack()) {
                log::error!("Debug power request failed");
                return Err(DapError::TargetPowerUpFailed.into());
            }

            // According to CMSIS docs, here's where we would clear errors
            // in ABORT, but we do that above instead.
        }

        Ok(())
    }

    /// Initialize core debug system. This is based on the
    /// `DebugCoreStart` function from the [ARM SVD Debug Description].
    ///
    /// [ARM SVD Debug Description]: http://www.keil.com/pack/doc/cmsis/Pack/html/debug_description.html#debugCoreStart
    #[doc(alias = "DebugCoreStart")]
    fn debug_core_start(
        &self,
        core: &mut Memory,
        core_type: CoreType,
        debug_base: Option<u64>,
        cti_base: Option<u64>,
    ) -> Result<(), crate::Error> {
        log::warn!("debug_core_start");
        // Dispatch based on core type (Cortex-A vs M)
        match core_type {
            CoreType::Armv7a => armv7a_core_start(core, debug_base),
            CoreType::Armv8a => armv8a_core_start(core, debug_base, cti_base),
            CoreType::Armv6m | CoreType::Armv7m | CoreType::Armv7em | CoreType::Armv8m => {
                cortex_m_core_start(core)
            }
            _ => panic!(
                "Logic inconsistency bug - non ARM core type passed {:?}",
                core_type
            ),
        }
    }

    /// Configure the target to stop code execution after a reset. After this, the core will halt when it comes
    /// out of reset. This is based on the `ResetCatchSet` function from
    /// the [ARM SVD Debug Description].
    ///
    /// [ARM SVD Debug Description]: http://www.keil.com/pack/doc/cmsis/Pack/html/debug_description.html#resetCatchSet
    #[doc(alias = "ResetCatchSet")]
    fn reset_catch_set(
        &self,
        core: &mut Memory,
        core_type: CoreType,
        debug_base: Option<u64>,
    ) -> Result<(), crate::Error> {
        log::warn!("reset_catch_set");
        // Dispatch based on core type (Cortex-A vs M)
        match core_type {
            CoreType::Armv7a => armv7a_reset_catch_set(core, debug_base),
            CoreType::Armv8a => armv8a_reset_catch_set(core, debug_base),
            CoreType::Armv6m | CoreType::Armv7m | CoreType::Armv7em | CoreType::Armv8m => {
                cortex_m_reset_catch_set(core)
            }
            _ => panic!(
                "Logic inconsistency bug - non ARM core type passed {:?}",
                core_type
            ),
        }
    }

    /// Free hardware resources allocated by ResetCatchSet.
    /// This is based on the `ResetCatchSet` function from
    /// the [ARM SVD Debug Description].
    ///
    /// [ARM SVD Debug Description]: http://www.keil.com/pack/doc/cmsis/Pack/html/debug_description.html#resetCatchClear
    #[doc(alias = "ResetCatchClear")]
    fn reset_catch_clear(
        &self,
        core: &mut Memory,
        core_type: CoreType,
        debug_base: Option<u64>,
    ) -> Result<(), crate::Error> {
        log::warn!("reset_catch_clear");
        // Dispatch based on core type (Cortex-A vs M)
        match core_type {
            CoreType::Armv7a => armv7a_reset_catch_clear(core, debug_base),
            CoreType::Armv8a => armv8a_reset_catch_clear(core, debug_base),
            CoreType::Armv6m | CoreType::Armv7m | CoreType::Armv7em | CoreType::Armv8m => {
                cortex_m_reset_catch_clear(core)
            }
            _ => panic!(
                "Logic inconsistency bug - non ARM core type passed {:?}",
                core_type
            ),
        }
    }

    /// Executes a system-wide reset without debug domain (or warm-reset that preserves debug connection) via software mechanisms,
    /// for example AIRCR.SYSRESETREQ.  This is based on the
    /// `ResetSystem` function from the [ARM SVD Debug Description].
    ///
    /// [ARM SVD Debug Description]: http://www.keil.com/pack/doc/cmsis/Pack/html/debug_description.html#resetSystem
    #[doc(alias = "ResetSystem")]
    fn reset_system(
        &self,
        interface: &mut Memory,
        core_type: CoreType,
        debug_base: Option<u64>,
    ) -> Result<(), crate::Error> {
        log::warn!("reset_system");
        // Dispatch based on core type (Cortex-A vs M)
        match core_type {
            CoreType::Armv7a => armv7a_reset_system(interface, debug_base),
            CoreType::Armv8a => armv8a_reset_system(interface, debug_base),
            CoreType::Armv6m | CoreType::Armv7m | CoreType::Armv7em | CoreType::Armv8m => {
                cortex_m_reset_system(interface)
            }
            _ => panic!(
                "Logic inconsistency bug - non ARM core type passed {:?}",
                core_type
            ),
        }
    }

    /// Check if the device is in a locked state and unlock it.
    /// Use query command elements for user confirmation.
    /// Executed after having powered up the debug port. This is based on the
    /// `DebugDeviceUnlock` function from the [ARM SVD Debug Description].
    ///
    /// [ARM SVD Debug Description]: http://www.keil.com/pack/doc/cmsis/Pack/html/debug_description.html#debugDeviceUnlock
    #[doc(alias = "DebugDeviceUnlock")]
    fn debug_device_unlock(
        &self,
        _interface: &mut Box<dyn ArmProbeInterface>,
        _default_ap: MemoryAp,
        _permissions: &crate::Permissions,
    ) -> Result<(), crate::Error> {
        log::warn!("debug_device_unlock");
        // Empty by default
        Ok(())
    }

    /// Executed before step or run command to support recovery from a lost target connection, e.g. after a low power mode.
    /// This is based on the `RecoverSupportStart` function from the [ARM SVD Debug Description].
    ///
    /// [ARM SVD Debug Description]: http://www.keil.com/pack/doc/cmsis/Pack/html/debug_description.html#recoverSupportStart
    #[doc(alias = "RecoverSupportStart")]
    fn recover_support_start(&self, _interface: &mut crate::Memory) -> Result<(), crate::Error> {
        log::warn!("recover_support_start");
        // Empty by default
        Ok(())
    }

    /// Executed when the debugger session is disconnected from the core.
    ///
    /// This is based on the `DebugCoreStop` function from the [ARM SVD Debug Description].
    ///
    /// [ARM SVD Debug Description]: http://www.keil.com/pack/doc/cmsis/Pack/html/debug_description.html#recoverSupportStart
    #[doc(alias = "DebugCoreStop")]
    fn debug_core_stop(
        &self,
        _interface: &mut Box<dyn ArmProbeInterface>,
    ) -> Result<(), crate::Error> {
        log::warn!("debug_core_stop");
        // Empty by default
        Ok(())
    }
}
