use anyhow::{anyhow, Context, Result};
use object::elf::FileHeader32;
use object::elf::PT_LOAD;
use object::read::elf::ProgramHeader;
use object::Endianness;
use object::ObjectSection;
use object::{Object, ObjectSegment, SegmentFlags};
use probe_rs::architecture::arm::ap::MemoryAp;
use probe_rs::architecture::arm::{ApAddress, ArmProbeInterface, DpAddress, Pins};
use probe_rs::config::MemoryRange;
use probe_rs::flashing::DownloadOptions;
use probe_rs::flashing::FlashLoader;
use probe_rs::Memory;
use probe_rs::{Permissions, Probe};
use std::fs::File;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct DataChunk {
    address: u32,
    segment_offset: u64,
    segment_filesize: u64,
}

pub struct Atsaml10(());

impl Atsaml10 {
    const DSU_ADDR: u32 = 0x41002100;
    const DSU_STATUSA_ADDR: u32 = Self::DSU_ADDR + 0x1;
    const DSU_STATUSB_ADDR: u32 = Self::DSU_ADDR + 0x2;
    const _DSU_DID_ADDR: u32 = Self::DSU_ADDR + 0x18;
    const DSU_BCC0_ADDR: u32 = Self::DSU_ADDR + 0x20;
    const DSU_BCC1_ADDR: u32 = Self::DSU_ADDR + 0x24;
    const NVMCTRL_ADDR: u32 = 0x41004000;
    const NVMCTRL_CTRLA_ADDR: u32 = Self::NVMCTRL_ADDR + 0x00;
    const NVMCTRL_CTRLC_ADDR: u32 = Self::NVMCTRL_ADDR + 0x08;
    const NVMCTRL_STATUS_ADDR: u32 = Self::NVMCTRL_ADDR + 0x18;
    const NVMCTRL_ADDR_ADDR: u32 = Self::NVMCTRL_ADDR + 0x1C;
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
    // Flash row size.
    const ROW_SIZE: u32 = 256;
    // Erase row command.
    const NVMCTRL_CTRLA_ER_CMD: u16 = 0xa502;
    // XXX Overwrite the first two bytes of CTRLB which default to 0...
    const NVMCTRL_CTRLA_ER_CMD32: u32 = (Self::NVMCTRL_CTRLA_ER_CMD as u32) << 16;
    const NVMCTRL_STATUS_READY: u8 = 1 << 2;

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
    let mut interface = probe.try_into_arm_interface().map_err(|(_, e)| e)?;

    /*
        // First, do a cold plug sequence.
        // XXX Something before this is sending commands...
        log::warn!("MANUAL cold plug");
        atsaml10
            .do_cold_plug(&mut interface)
            .context("Failed to do cold plug")?;
    */

    // Don't know how to call this as a function...
    {
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
    }

    log::warn!("MANUAL initialize");
    let mut interface = interface.initialize_unspecified()?;

    log::warn!("MANUAL port");
    let port = ApAddress {
        dp: DpAddress::Default,
        ap: 0,
    };

    let default_memory_ap = MemoryAp::new(port);

    /*
    log::warn!("MANUAL read_raw_ap_register");
    let val = interface.read_raw_ap_register(port, 0)?;
    log::warn!("val {}", val);
    */

    let atsaml10 = Atsaml10(());

    log::warn!("MANUAL memory_interface");
    // This runs a bunch of commands, but none of them seem to error.
    let mut memory = interface.memory_interface(default_memory_ap)?;

    // Now follow the CMD_EXIT to Park mode diagram (14-9 page 81).
    log::warn!("MANUAL exit_reset_extension");
    atsaml10.exit_reset_extension(&mut memory)?;
    log::warn!("MANUAL exit_interactive_mode");
    // By not clearing BREXT here we go into park mode.
    atsaml10.exit_interactive_mode(&mut memory)?;

    // Make sure the debug access level is 2 (unlocked).
    log::warn!("MANUAL read DAL");
    let dal = atsaml10.get_dal(&mut memory)?;
    if dal != 2 {
        return Err(anyhow!(
            "Device is locked. DAL == {} != 2. Try erasing the chip to clear.",
            dal
        ));
    }

    // Get the target definition.
    let target = probe_rs::config::get_target_by_name("ATSAML10E16A")?;

    // Create a flash loader.
    let mut loader = FlashLoader::new(target.memory_map.to_vec(), target.source().clone());

    // Add data to the flash loader from an ELF file.
    let elf_path = Path::new("/Users/dwatson/work/fridge_monitor/src/fridge_sensor_rs/target/thumbv8m.base-none-eabi/release/blink");
    let mut file = File::open(&elf_path)?;
    loader.load_elf_data(&mut file)?;

    // Manually implement the flasher.

    // First figure out how to read the data from the elf file.
    let bin_data = std::fs::read("/Users/dwatson/work/fridge_monitor/src/fridge_sensor_rs/target/thumbv8m.base-none-eabi/release/blink")?;
    let obj_file = object::read::elf::ElfFile::<FileHeader32<Endianness>>::parse(&*bin_data)?;

    let endian = obj_file.endian();
    println!("Endian {:?}", endian);

    let mut extracted_data = Vec::new();

    for segment in obj_file.raw_segments() {
        let p_type = segment.p_type(endian);
        let p_paddr = segment.p_paddr(endian);
        let p_vaddr = segment.p_vaddr(endian);

        let segment_data = segment
            .data(endian, &*bin_data)
            .map_err(|_| anyhow!("Failed to access data in segment"))?;

        if segment_data.is_empty() || p_type != PT_LOAD {
            continue;
        }
        println!(
            "Loadable Segment physical {:x}, virtual {:x}",
            p_paddr, p_vaddr
        );

        let (segment_offset, segment_filesize) = segment.file_range(endian);

        let sector: core::ops::Range<u64> = segment_offset..segment_offset + segment_filesize;

        let mut found = false;
        for section in obj_file.sections() {
            let (section_offset, section_filesize) = match section.file_range() {
                Some(range) => range,
                None => continue,
            };
            if sector.contains_range(&(section_offset..section_offset + section_filesize)) {
                println!("Matching section: {:?}", section.name()?);
                found = true;
            }
        }

        if !found {
            println!("No matching sections found!");
            continue;
        }

        extracted_data.push(DataChunk {
            address: p_paddr,
            segment_offset,
            segment_filesize,
        });
    }

    extracted_data.sort();

    let mut flash_data = Vec::new();
    for chunk in extracted_data {
        match flash_data.pop() {
            None => flash_data.push(chunk),
            Some(prev) => {
                let prev_len: u32 = prev.segment_filesize.try_into()?;
                let next_addr = prev.address + prev_len;
                if next_addr == chunk.address {
                    flash_data.push(DataChunk {
                        address: prev.address,
                        segment_offset: prev.segment_offset,
                        segment_filesize: prev.segment_filesize + chunk.segment_filesize,
                    });
                } else {
                    flash_data.push(prev);
                    flash_data.push(chunk);
                }
            }
        }
    }
    // XXX Probably need to check for segments with holes within a row...

    let row_size: usize = Atsaml10::ROW_SIZE as usize;

    // Actually do the flash.
    for chunk in flash_data {
        let data = &bin_data[chunk.segment_offset as usize..][..chunk.segment_filesize as usize];
        hexdump::hexdump(data);

        let size: u32 = chunk.segment_filesize.try_into()?;

        // Enable automatic writes.
        memory.write_word_8((Atsaml10::NVMCTRL_CTRLC_ADDR).into(), 0)?;

        let mut addr = chunk.address;

        for row in data.chunks(row_size) {
            // Set the address.
            memory.write_word_32((Atsaml10::NVMCTRL_ADDR_ADDR).into(), addr)?;

            // Clear memory row.
            // XXX Would prefer to write this as a 16 bit addr...
            memory.write_word_32(
                (Atsaml10::NVMCTRL_CTRLA_ADDR).into(),
                Atsaml10::NVMCTRL_CTRLA_ER_CMD32,
            )?;

            // Wait for the NVM controller to be ready.
            loop {
                let status = memory.read_word_8((Atsaml10::NVMCTRL_STATUS_ADDR).into())?;
                if (status & Atsaml10::NVMCTRL_STATUS_READY) != 0 {
                    break;
                }
            }

            if row.len() < row_size {
                let mut buf = Vec::with_capacity(row_size);
                buf.extend_from_slice(row);
                buf.resize(row_size, 0xff);

                memory.write_8(addr.into(), &buf)?;
            } else {
                memory.write_8(addr.into(), row)?;
            }

            addr += Atsaml10::ROW_SIZE;
        }
    }

    let _opt = DownloadOptions::default();
    // XXX I can't figure out how to create a session...
    // loader.commit(interface, opt)?;

    log::warn!("MANUAL exit");
    Ok(())
}
