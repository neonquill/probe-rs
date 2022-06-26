```
dwatson@cripps-pink ~/D/tmp> nm keil/flash/ATSAML10_64.FLM | grep T
00000000 A BuildAttributes$$THM_ISAv3M$S$PE$A:L22$X:L11$S22$IEEE1$IW$RWPI$~STKCKD$USESV7$~SHL$OSPACE$ROPI$EBA8$STANDARDLIB$REQ8$PRES8$EABIv2
0000003e T EraseSector
00000000 T Init
0000005e T ProgramPage
0000003a T UnInit
000000e4 T __aeabi_uread4
000000e4 T __rt_uread4
```

```
dwatson@cripps-pink ~/D/tmp> less Microchip.SAML10_DFP.pdsc
         <device Dname="ATSAML10D16A">
            <processor Dcore="Cortex-M23"
                       Dendian="Little-endian"
                       Dmpu="MPU"
                       Dfpu="NO_FPU"/>
            <compile header="include/sam.h" define="__SAML10D16A__"/>
            <debug svd="svd/ATSAML10D16A.svd"/>
            <memory id="IROM1"
                    start="0x00000000"
                    size="0x10000"
                    default="1"
                    startup="1"/>
            <memory id="IROM2" start="0x00400000" size="0x800"/>
            <memory id="IRAM1" start="0x20000000" size="0x4000" default="1"/>
            <algorithm name="keil/flash/ATSAML10_64.FLM"
                       start="0x00000000"
                       size="0x10000"
                       default="1"/>
```

https://probe.rs/docs/knowledge-base/cmsis-packs/#yaml-format

So the instructions value is just some dump of the elf FLM file.
And the `pc_*` values are just the offsets of those symbols (+1?) into that
code.

Architecture specific code on top of the config files:
```
dwatson@cripps-pink ~/w/t/probe-rs (master)> ls probe-rs/src/architecture/arm/sequences/
mod.rs   nrf53.rs nxp.rs   stm32.rs
```

Code that switches to the custom sequence in `probe-rs/src/config/target.rs`:
```rust
        // We always just take the architecture of the first core which is okay if there is no mixed architectures.
        let mut debug_sequence = match chip.cores[0].core_type.architecture() {
            Architecture::Arm => DebugSequence::Arm(DefaultArmSequence::create()),
            Architecture::Riscv => DebugSequence::Riscv(DefaultRiscvSequence::create()),
        };

        if chip.name.starts_with("LPC55S16") || chip.name.starts_with("LPC55S69") {
            log::warn!("Using custom sequence for LPC55S16/LPC55S69");
            debug_sequence = DebugSequence::Arm(LPC55S69::create());
        } else if chip.name.starts_with("esp32c3") {
            log::warn!("Using custom sequence for ESP32c3");
            debug_sequence = DebugSequence::Riscv(ESP32C3::create());
        } else if chip.name.starts_with("nRF5340") {
            log::warn!("Using custom sequence for nRF5340");
            debug_sequence = DebugSequence::Arm(Nrf5340::create());
        } else if chip.name.starts_with("STM32H7") {
            log::warn!("Using custom sequence for STM32H7");
            debug_sequence = DebugSequence::Arm(Stm32h7::create());
        }
```

Example code in /Users/dwatson/.mchp_packs/Microchip/SAML10_DFP/3.9.116/scripts/dap_cortex-m23.py

To trigger the reset on start:
cargo flash --release --chip ATSAML10E16A --elf target/thumbv8m.base-none-eabi/release/blink --connect-under-reset

But the pins never change, I'm doing something wrong here.


This attempt to toggle the pins worked
Look at `~/Desktop/probe-rs-debug_success.sal` for the logic trace
```
dwatson@cripps-pink ~/w/t/probe-rs (master) [101]> cargo run --bin probe-rs-debug
warning: unused variable: `permissions`
  --> probe-rs/src/architecture/arm/sequences/atsam.rs:97:9
   |
97 |         permissions: &crate::Permissions,
   |         ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_permissions`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: `probe-rs` (lib) generated 1 warning
    Finished dev [unoptimized + debuginfo] target(s) in 0.28s
     Running `target/debug/probe-rs-debug`
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap::tools: Searching for CMSIS-DAP probes using libusb
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:68] DAPLink CMSIS-DAP: CMSIS-DAP device with 5 interfaces
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:80] Interface 0 is not HID, skipping
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:80] Interface 1 is not HID, skipping
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:80] Interface 2 is not HID, skipping
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:96]   Interface 3: CMSIS-DAP v1
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:106] Will use interface number 3 for CMSIS-DAPv1
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap::tools: Found 1 CMSIS-DAP probes using libusb, searching HID
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:128] CMSIS-DAP device with USB path: "DevSrvsID:4295092112"
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:129]                 product_string: "DAPLink CMSIS-DAP"
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:130]                      interface: 3
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:40] Ignoring duplicate DAPLink CMSIS-DAP (VID: 0d28, PID: 0204, Serial: 0409170280f408bc00000000000000000000000097969906, CmsisDap)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap::tools: Found 1 CMSIS-DAP probes total
20:49:52 [DEBUG] (1) jaylink: libusb 1.0.26.11724
20:49:52 [DEBUG] (1) jaylink: libusb has capability API: true
20:49:52 [DEBUG] (1) jaylink: libusb has HID access: false
20:49:52 [DEBUG] (1) jaylink: libusb has hotplug support: true
20:49:52 [DEBUG] (1) jaylink: libusb can detach kernel driver: true
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:257] Attempting to open device matching 0d28:0204:0409170280f408bc00000000000000000000000097969906
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:273] Trying device Bus 000 Device 001: ID 21a9:1005
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:273] Trying device Bus 020 Device 005: ID 0d28:0204
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:68] DAPLink CMSIS-DAP: CMSIS-DAP device with 5 interfaces
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:80] Interface 0 is not HID, skipping
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:80] Interface 1 is not HID, skipping
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:80] Interface 2 is not HID, skipping
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:96]   Interface 3: CMSIS-DAP v1
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:106] Will use interface number 3 for CMSIS-DAPv1
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap::tools: Could not open 0d28:0204 in CMSIS-DAP v2 mode
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::tools: [probe-rs/src/probe/cmsisdap/tools.rs:273] Trying device Bus 020 Device 004: ID 1915:c00a
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap::tools: Attempting to open 0d28:0204 in CMSIS-DAP v1 mode
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap::commands: Draining probe of any pending data.
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap::commands: Attempt 1 to find packet size
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 00, FF, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [00, 02, 40, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap::commands: Success: packet size is 64
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap::commands: Configuring probe to use packet size 64
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 00, FE, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [00, 01, 04, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 00, F0, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [00, 01, 31, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Detected probe capabilities: Capabilities { _swd_implemented: true, _jtag_implemented: false, swo_uart_implemented: false, swo_manchester_implemented: false, _atomic_commands_implemented: true, _test_domain_timer_implemented: true, swo_streaming_trace_implemented: false, _uart_communication_port_implemented: false, uart_com_port_implemented: false }
20:49:52 [DEBUG] (1) probe_rs::config::registry: Searching registry for chip with name ATSAML10E16A
20:49:52 [DEBUG] (1) probe_rs::config::registry: Exact match for chip name: ATSAML10E16A
20:49:52 [WARN] Using custom sequence for ATSAM10
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attaching to target system (clock = 1000kHz)
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 02, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [02, 01, 00]...
20:49:52 [INFO] Using protocol SWD
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 11, 40, 42, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [11, 00, 42, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 04, 00, FF, FF, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [04, 00, FF, FF, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 13, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [13, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 01, 00, 01, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [01, 00, 01, 00]...
20:49:52 [WARN] debug_port_setup
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 12, 33, FF, FF, FF, FF, FF, FF, 07, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [12, 00, FF, FF, FF, FF, FF, FF, 07, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 12, 10, 9E, E7, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [12, 00, 9E, E7, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 12, 33, FF, FF, FF, FF, FF, FF, 07, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [12, 00, FF, FF, FF, FF, FF, FF, 07, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 12, 03, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [12, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=DebugPort, addr=0)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 1 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 1 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 01, 02, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 01, 01, 77, 14, F1, 0B, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 1 of batch of 1 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::communication_interface: Selecting DP Default
20:49:52 [WARN] debug_port_start
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Writing DP register ABORT, value=0x0000001e
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=DebugPort, addr=0, data=0x0000001e
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Writing DP register SELECT, value=0x00000000
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=DebugPort, addr=8, data=0x00000000
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Reading DP register CTRL/STAT
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=DebugPort, addr=4)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 3 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 03, 00, 1E, 00, 00, 00, 08, 00, 00, 00, 00, 06, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 03, 01, 00, 00, 00, 00, 00, 08, 00, 00, 00, 00, 06, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 of batch of 3 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Read    DP register CTRL/STAT, value=0x00000000
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Writing DP register CTRL/STAT, value=0x50000000
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=DebugPort, addr=4, data=0x50000000
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Reading DP register CTRL/STAT
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=DebugPort, addr=4)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 2 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 02, 04, 00, 00, 00, 50, 06, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 02, 01, 00, 00, 00, F0, 50, 06, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 of batch of 2 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Read    DP register CTRL/STAT, value=0xf0000000
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Writing DP register CTRL/STAT, value=0x50000f00
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=DebugPort, addr=4, data=0x50000f00
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Reading DP register CTRL/STAT
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=DebugPort, addr=4)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 2 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 02, 04, 00, 0F, 00, 50, 06, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 02, 01, 00, 00, 00, F0, 50, 06, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 of batch of 2 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Read    DP register CTRL/STAT, value=0xf0000000
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Reading DP register CTRL/STAT
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=DebugPort, addr=4)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 1 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 1 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 01, 06, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 01, 01, 00, 00, 00, F0, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 1 of batch of 1 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Read    DP register CTRL/STAT, value=0xf0000000
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Writing DP register CTRL/STAT, value=0xf0000000
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=DebugPort, addr=4, data=0xf0000000
20:49:52 [TRACE] (1) probe_rs::architecture::arm::communication_interface: [probe-rs/src/architecture/arm/communication_interface.rs:456] Searching valid APs
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register IDR
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::communication_interface: Changing AP to 0, AP_BANK_SEL to 15
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Writing DP register SELECT, value=0x000000f0
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=DebugPort, addr=8, data=0x000000f0
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=252)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 3 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 03, 04, 00, 00, 00, F0, 08, F0, 00, 00, 00, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 03, 01, 25, 00, 77, 04, F0, 08, F0, 00, 00, 00, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 of batch of 3 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    IDR, value=0x4770025
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register IDR
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::communication_interface: Changing AP to 1, AP_BANK_SEL to 15
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Writing DP register SELECT, value=0x010000f0
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=DebugPort, addr=8, data=0x010000f0
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=252)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 2 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 02, 08, F0, 00, 00, 01, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 02, 01, 00, 00, 00, 00, 01, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 of batch of 2 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    IDR, value=0x0
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register IDR
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::communication_interface: Changing AP to 0, AP_BANK_SEL to 15
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Writing DP register SELECT, value=0x000000f0
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=DebugPort, addr=8, data=0x000000f0
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=252)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 2 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 02, 08, F0, 00, 00, 00, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 02, 01, 25, 00, 77, 04, 00, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 of batch of 2 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    IDR, value=0x4770025
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register BASE
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=248)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 1 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 1 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 01, 0B, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 01, 01, 03, 30, 00, 41, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 1 of batch of 1 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    BASE, value=0x41003003
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register BASE2
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=240)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 1 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 1 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 01, 03, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 01, 01, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 1 of batch of 1 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    BASE2, value=0x0
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register CSW
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::communication_interface: Changing AP to 0, AP_BANK_SEL to 0
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Writing DP register SELECT, value=0x00000000
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=DebugPort, addr=8, data=0x00000000
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=0)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 2 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 02, 08, 00, 00, 00, 00, 03, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 02, 01, 40, 00, 00, 43, 00, 03, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 of batch of 2 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    CSW, value=0x43000040
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register CSW, value=CSW { DbgSwEnable: 1, HNONSEC: 1, PROT: 6, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U8 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=0, data=0xe3000010
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register CSW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=0)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 2 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 02, 01, 10, 00, 00, E3, 03, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 02, 01, 50, 00, 00, 43, E3, 03, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 of batch of 2 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    CSW, value=0x43000050
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 1, PROT: 0, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 1, AddrInc: Off, _RES1: 0, SIZE: U8 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=0, data=0x43000040
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::communication_interface: HNONSEC supported: true
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register CFG
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::communication_interface: Changing AP to 0, AP_BANK_SEL to 15
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Writing DP register SELECT, value=0x000000f0
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=DebugPort, addr=8, data=0x000000f0
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=244)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 3 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 03, 01, 40, 00, 00, 43, 08, F0, 00, 00, 00, 07, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 03, 01, 00, 00, 00, 00, 43, 08, F0, 00, 00, 00, 07, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 of batch of 3 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    CFG, value=0x0
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::communication_interface: AP GenericAp { address: ApAddress { dp: Default, ap: 0 } }: MemoryAp(MemoryApInformation { address: ApAddress { dp: Default, ap: 0 }, only_32bit_data_size: false, debug_base_address: 1090531328, supports_hnonsec: true, has_large_address_extension: false, has_large_data_extension: false })
20:49:52 [WARN] XXX
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::communication_interface: Changing AP to 0, AP_BANK_SEL to 0
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::dp: Writing DP register SELECT, value=0x00000000
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=DebugPort, addr=8, data=0x00000000
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: 41002118 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002118
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register DRW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=12)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 4 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 4 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 04, 08, 00, 00, 00, 00, 01, 12, 00, 00, 23, 05, 18, 21, 00, 41, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 04, 01, 00, 01, 84, 20, 00, 01, 12, 00, 00, 23, 05, 18, 21, 00, 41, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 4 of batch of 4 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    DRW, value=0x20840100
20:49:52 [WARN] Read DID 20840100
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U8 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000010
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: 41002102 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register DRW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=12)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 3 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 03, 01, 10, 00, 00, 23, 05, 02, 21, 00, 41, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 03, 01, 00, 00, 8E, 08, 23, 05, 02, 21, 00, 41, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 of batch of 3 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    DRW, value=0x88e0000
20:49:52 [WARN] Read STATUSB 8e
20:49:52 [WARN] DAL 2
20:49:52 [WARN] debug_core_start
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: e000edf0 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register DRW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=12)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 3 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 03, 01, 12, 00, 00, 23, 05, F0, ED, 00, E0, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 03, 01, 00, 00, 10, 03, 23, 05, F0, ED, 00, E0, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 of batch of 3 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    DRW, value=0x3100000
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: e000edf0 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register DRW, value=DRW { data: a05f0001 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=12, data=0xa05f0001
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: e000edf0 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register DRW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=12)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 5 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 5 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 05, 05, F0, ED, 00, E0, 0D, 01, 00, 5F, A0, 01, 12, 00, 00, 23, 05, F0, ED, 00, E0, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 05, 01, 01, 00, 10, 01, E0, 0D, 01, 00, 5F, A0, 01, 12, 00, 00, 23, 05, F0, ED, 00, E0, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 5 of batch of 5 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    DRW, value=0x1100001
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::core::armv8m: State when connecting: Dhcsr { .0: 1100001, s_restart_st: false, s_reset_st: false, s_retire_st: true, s_fpd: false, s_suide: false, s_nsuide: false, s_sde: true, s_lockup: false, s_sleep: false, s_halt: false, s_regrdy: false, c_pmov: false, c_snapstall: false, c_maskints: false, c_step: false, c_halt: false, c_debugen: true }
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: e000ed30 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000ed30
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register DRW, value=DRW { data: 1f }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=12, data=0x0000001f
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: e0002000 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0xe0002000
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register DRW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=12)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 4 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 4 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 04, 05, 30, ED, 00, E0, 0D, 1F, 00, 00, 00, 05, 00, 20, 00, E0, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 04, 01, 40, 00, 00, 10, E0, 0D, 1F, 00, 00, 00, 05, 00, 20, 00, E0, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 4 of batch of 4 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    DRW, value=0x10000040
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: e0002008 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0xe0002008
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register DRW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=12)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 2 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 02, 05, 08, 20, 00, E0, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 02, 01, FE, FF, FB, FF, E0, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 of batch of 2 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    DRW, value=0xfffbfffe
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: e000200c }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000200c
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register DRW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=12)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 2 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 02, 05, 0C, 20, 00, E0, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 02, 01, FE, FD, FF, FB, E0, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 of batch of 2 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    DRW, value=0xfbfffdfe
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: e0002010 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0xe0002010
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register DRW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=12)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 2 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 02, 05, 10, 20, 00, E0, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 02, 01, FE, FE, FF, FF, E0, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 of batch of 2 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    DRW, value=0xfffffefe
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: e0002014 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0xe0002014
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register DRW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=12)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 2 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 02, 05, 14, 20, 00, E0, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 02, 01, FE, FF, FE, FF, E0, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 2 of batch of 2 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:209] Transfer status: ACK
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Read register    DRW, value=0xfffefffe
20:49:52 [WARN] XXX cmsisdap:swj_pins request out: 131, select: 131, wait: 0
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 10, 83, 83, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [10, 83, 83, 00]...
20:49:52 [WARN] XXX cmsisdap:swj_pins response: 131
Value: 131
20:49:52 [WARN] XXX cmsisdap:swj_pins request out: 0, select: 131, wait: 0
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 10, 00, 83, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [10, 00, 83, 00]...
20:49:52 [WARN] XXX cmsisdap:swj_pins response: 0
Value: 0
20:49:52 [WARN] XXX cmsisdap:swj_pins request out: 131, select: 131, wait: 0
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 10, 83, 83, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [10, 83, 83, 00]...
20:49:52 [WARN] XXX cmsisdap:swj_pins response: 131
Value: 131
Hello, world!
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: e0002000 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0xe0002000
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register DRW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=12)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 3 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 03, 01, 12, 00, 00, 23, 05, 00, 20, 00, E0, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 00, 07, 01, 12, 00, 00, 23, 05, 00, 20, 00, E0, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 0 of batch of 3 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:213] Transfer status: NACK
20:49:52 [WARN] Could not clear all hardware breakpoints: ArchitectureSpecific(RegisterRead { address: 12, name: "DRW", source: ArchitectureSpecific(NoAcknowledge) })
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Writing register TAR, value=TAR { address: e000edfc }
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edfc
20:49:52 [DEBUG] (1) probe_rs::architecture::arm::ap: Reading register DRW
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Adding command to batch: Read(port=AccessPort, addr=12)
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 3 items in batch
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Attempting batch of 3 items
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 05, 00, 03, 01, 12, 00, 00, 23, 05, FC, ED, 00, E0, 0F, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [05, 00, 07, 01, 12, 00, 00, 23, 05, FC, ED, 00, E0, 0F, 00]...
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: 0 of batch of 3 items suceeded
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap: [probe-rs/src/probe/cmsisdap/mod.rs:213] Transfer status: NACK
20:49:52 [WARN] Could not stop core tracing: ArchitectureSpecific(RegisterRead { address: 12, name: "DRW", source: ArchitectureSpecific(NoAcknowledge) })
20:49:52 [WARN] debug_core_stop
20:49:52 [DEBUG] (1) probe_rs::probe::cmsisdap: Detaching from CMSIS-DAP probe
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 03, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [03, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 01, 00]...
20:49:52 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [01, 00]...
```

Looks like the magic command I want is in:
probe-rs/src/probe/cmsisdap/mod.rs

```
impl DebugProbe for CmsisDap {
  fn attach(...
```

It sends the `DAP_Connect` command to connect
```
21:23:56 [DEBUG] (1) probe_rs::probe::cmsisdap: Attaching to target system (clock = 1000kHz)
21:23:56 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 02, 00]...
21:23:56 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [02, 01, 00]...
```

then `DAP_SWJ_Clock` to set the clock frequency:
```
21:23:56 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 11, 40, 42, 0F, 00]...
21:23:56 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [11, 00, 42, 0F, 00]...
```

then `DAP_TransferConfigure`
```
21:23:56 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 04, 00, FF, FF, 00]...
21:23:56 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [04, 00, FF, FF, 00]...
```

then `DAP_SWD_Configure`
```
21:23:56 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 13, 00]...
21:23:56 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [13, 00]...
```
and then the `DAP_HostStatus` command to turn on the LED.
```
21:23:56 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Transmit buffer: [00, 01, 00, 01, 00]...
21:23:56 [TRACE] (1) probe_rs::probe::cmsisdap::commands: [probe-rs/src/probe/cmsisdap/commands/mod.rs:393] Receive buffer: [01, 00, 01, 00]...
```

This is all called from:
`probe-rs/src/sessions.rs`
```
impl Session {
  fn new(
```

Specifically `probe.inner_attach()?;` seems to be the trigger.

writing the sample code and turning on trace debugging was the magic solution
now I could follow where things where happening in the code

Switched to modifying `debug_port_setup()` to do a cold connection and things worked fine


## 18 June 2022

Current state of the code:
```
dwatson@cripps-pink ~/w/f/s/fridge_sensor_rs (master) [1]> cargo flash --release --chip ATSAML10E16A --elf target/thumbv8m.base-none-eabi/release/blink --connect-under-reset
    Flashing target/thumbv8m.base-none-eabi/release/blink
        WARN probe_rs::config::target > Using custom sequence for ATSAM10
        WARN probe_rs::session        > XXXA
        WARN probe_rs::architecture::arm::sequences::atsam > atsaml10 reset_hardware_assert
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 131
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 0, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 3
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 0, select: 129, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 2
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 130
        WARN probe_rs::session                             > XXXB
        WARN probe_rs::session                             > XXXC
        WARN probe_rs::architecture::arm::sequences        > debug_port_setup
        WARN probe_rs::session                             > XXXD
        WARN probe_rs::architecture::arm::sequences        > debug_port_start
        WARN probe_rs::architecture::arm::sequences::atsam > XXX
        WARN probe_rs::architecture::arm::sequences::atsam > Read DID 20840100
        WARN probe_rs::architecture::arm::sequences::atsam > Read STATUSB 4
        WARN probe_rs::architecture::arm::sequences::atsam > DAL 0
        WARN probe_rs::architecture::arm::sequences::atsam > Chip is locked (debug access level: DAL == 2)
        WARN probe_rs::architecture::arm::sequences::atsam > Need to allow chip erase to continue.
        WARN probe_rs::architecture::arm::sequences::atsam > Going to erase anyway!!!!
        WARN probe_rs::architecture::arm::sequences::atsam > Erasing chip
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa1
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa2
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa3
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa4
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa5
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa6
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa7
        WARN probe_rs::architecture::arm::sequences::atsam > Received status update after 1 cycles
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa8
        WARN probe_rs::architecture::arm::sequences::atsam > XXX Chip Erase succeeded
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa9
        WARN probe_rs::architecture::arm::sequences::atsam > XXXaa
        WARN probe_rs::architecture::arm::sequences::atsam > Failed to exit to park!: status 3959423013
        WARN probe_rs::architecture::arm::sequences::atsam > Chip unlock succeeded!
        WARN probe_rs::session                             > XXXE
        WARN probe_rs::architecture::arm::sequences        > debug_core_start
       Error Connecting to the chip was unsuccessful.

  Caused by:
          0: A core architecture specific error occurred
          1: Failed to read register DRW at address 0x0000000c
          2: An error specific to the selected architecture occurred
          3: Target device did not respond to request.
```

Not a big surprise here, the exit to park implementation is a bit sketchy.

Useful links to keep track of:

Generic programmer code:
https://github.com/ARMmbed/DAPLink

Generic CMSIS-DAP communication documentation:
https://open-cmsis-pack.github.io/Open-CMSIS-Pack-Spec/main/html/debug_description.html


Ok, two things don't make sense.
First, even though the chip erase appears to be working, I don't ever
end up back in an unlocked state, that's weird. Not only does it say
that the chip erase worked, but the board doesn't appear to blink any
more, so I think the code has been wiped.

Second, why am I getting an invalid command error when trying to exit
to park? Looking at the valid trace, there appears to be a reset happening
between the erase command and the exit command, which seems weird.
Also, the diagram in the docs seems to indicate that the reset is part
of the command?

## 19 June 2022

```
dwatson@cripps-pink ~/w/f/s/fridge_sensor_rs (master)> cargo flash --release --chip ATSAML10E16A --elf target/thumbv8m.base-none-eabi/release/blink --connect-under-reset
    Flashing target/thumbv8m.base-none-eabi/release/blink
        WARN probe_rs::config::target > Using custom sequence for ATSAM10
        WARN probe_rs::session        > XXXA
        WARN probe_rs::architecture::arm::sequences::atsam > atsaml10 reset_hardware_assert
        WARN probe_rs::architecture::arm::sequences::atsam > atsaml10 do_cold_plug()
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 131
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 0, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 3
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 0, select: 129, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 2
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 130
        WARN probe_rs::session                             > XXXB
        WARN probe_rs::session                             > XXXC
        WARN probe_rs::architecture::arm::sequences        > debug_port_setup
        WARN probe_rs::session                             > XXXD
        WARN probe_rs::architecture::arm::sequences        > debug_port_start
        WARN probe_rs::architecture::arm::sequences::atsam > XXX
        WARN probe_rs::architecture::arm::sequences::atsam > Read DID 20840100
        WARN probe_rs::architecture::arm::sequences::atsam > Read STATUSB 4
        WARN probe_rs::architecture::arm::sequences::atsam > DAL 0
        WARN probe_rs::architecture::arm::sequences::atsam > Chip is locked (debug access level: DAL == 0)
        WARN probe_rs::architecture::arm::sequences::atsam > Need to allow chip erase to continue.
        WARN probe_rs::architecture::arm::sequences::atsam > Going to erase anyway!!!!
        WARN probe_rs::architecture::arm::sequences::atsam > Erasing chip
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa1
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa2
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa3
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa4
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa5
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa6
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa7
        WARN probe_rs::architecture::arm::sequences::atsam > Received status update after 1 cycles
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa8
        WARN probe_rs::architecture::arm::sequences::atsam > XXX Chip Erase succeeded
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa9
        WARN probe_rs::architecture::arm::sequences::atsam > atsaml10 do_cold_plug()
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 131
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 0, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 3
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 0, select: 129, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 2
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 130
       Error Connecting to the chip was unsuccessful.

  Caused by:
          0: A core architecture specific error occurred
          1: Failed to read register DRW at address 0x0000000c
          2: An error specific to the selected architecture occurred
          3: Target device did not respond to request.
```

Latest attempt. I seem to be able to still control the pins, but I
need to re-run the init code.

## 22 June 2022

```
dwatson@cripps-pink ~/w/f/s/fridge_sensor_rs (master)> cargo flash --release --chip ATSAML10E16A --elf target/thumbv8m.base-none-eabi/release/blink --connect-under-reset
    Flashing target/thumbv8m.base-none-eabi/release/blink
        WARN probe_rs::config::target > Using custom sequence for ATSAM10
        WARN probe_rs::session        > XXXA
        WARN probe_rs::architecture::arm::sequences::atsam > atsaml10 reset_hardware_assert
        WARN probe_rs::architecture::arm::sequences::atsam > atsaml10 do_cold_plug()
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 131
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 0, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 3
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 0, select: 129, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 2
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 130
        WARN probe_rs::session                             > XXXB
        WARN probe_rs::session                             > XXXC
        WARN probe_rs::architecture::arm::sequences        > debug_port_setup
        WARN probe_rs::session                             > XXXD
        WARN probe_rs::architecture::arm::sequences        > debug_port_start
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa1
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa2
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa3
        WARN probe_rs::architecture::arm::sequences::atsam > XXXa4
        WARN probe_rs::architecture::arm::sequences::atsam > XXXaa
        WARN probe_rs::session                             > XXXE
        WARN probe_rs::architecture::arm::sequences        > debug_core_start
        WARN probe_rs::session                             > XXXF
        WARN probe_rs::architecture::arm::sequences        > reset_catch_set
        WARN probe_rs::architecture::arm::sequences        > reset_hardware_deassert
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 131
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 131
        WARN probe_rs::architecture::arm::sequences        > debug_core_stop
       Error Connecting to the chip was unsuccessful.

  Caused by:
          0: An error with the usage of the probe occurred
          1: Operation timed out
```

Not sure what went wrong...


Same thing, but with trace log level (beginning is missing):
```
       DEBUG probe_rs::config::registry > Exact match for chip name: ATSAML10E16A
        WARN probe_rs::config::target   > Using custom sequence for ATSAM10
       DEBUG probe_rs::probe::cmsisdap::tools > Searching for CMSIS-DAP probes using libusb
       TRACE probe_rs::probe::cmsisdap::tools > DAPLink CMSIS-DAP: CMSIS-DAP device with 5 interfaces
       TRACE probe_rs::probe::cmsisdap::tools > Interface 0 is not HID, skipping
       TRACE probe_rs::probe::cmsisdap::tools > Interface 1 is not HID, skipping
       TRACE probe_rs::probe::cmsisdap::tools > Interface 2 is not HID, skipping
       TRACE probe_rs::probe::cmsisdap::tools >   Interface 3: CMSIS-DAP v1
       TRACE probe_rs::probe::cmsisdap::tools > Will use interface number 3 for CMSIS-DAPv1
       DEBUG probe_rs::probe::cmsisdap::tools > Found 1 CMSIS-DAP probes using libusb, searching HID
       TRACE probe_rs::probe::cmsisdap::tools > CMSIS-DAP device with USB path: "DevSrvsID:4295156326"
       TRACE probe_rs::probe::cmsisdap::tools >                 product_string: "DAPLink CMSIS-DAP"
       TRACE probe_rs::probe::cmsisdap::tools >                      interface: 3
       TRACE probe_rs::probe::cmsisdap::tools > Ignoring duplicate DAPLink CMSIS-DAP (VID: 0d28, PID: 0204, Serial: 0409170280f408bc00000000000000000000000097969906, CmsisDap)
       DEBUG probe_rs::probe::cmsisdap::tools > Found 1 CMSIS-DAP probes total
       DEBUG jaylink                          > libusb 1.0.26.11724
       DEBUG jaylink                          > libusb has capability API: true
       DEBUG jaylink                          > libusb has HID access: false
       DEBUG jaylink                          > libusb has hotplug support: true
       DEBUG jaylink                          > libusb can detach kernel driver: true
       TRACE probe_rs::probe::cmsisdap::tools > Attempting to open device matching 0d28:0204:0409170280f408bc00000000000000000000000097969906
       TRACE probe_rs::probe::cmsisdap::tools > Trying device Bus 000 Device 002: ID 21a9:1005
       TRACE probe_rs::probe::cmsisdap::tools > Trying device Bus 020 Device 030: ID 0d28:0204
       TRACE probe_rs::probe::cmsisdap::tools > DAPLink CMSIS-DAP: CMSIS-DAP device with 5 interfaces
       TRACE probe_rs::probe::cmsisdap::tools > Interface 0 is not HID, skipping
       TRACE probe_rs::probe::cmsisdap::tools > Interface 1 is not HID, skipping
       TRACE probe_rs::probe::cmsisdap::tools > Interface 2 is not HID, skipping
       TRACE probe_rs::probe::cmsisdap::tools >   Interface 3: CMSIS-DAP v1
       TRACE probe_rs::probe::cmsisdap::tools > Will use interface number 3 for CMSIS-DAPv1
       DEBUG probe_rs::probe::cmsisdap::tools > Could not open 0d28:0204 in CMSIS-DAP v2 mode
       TRACE probe_rs::probe::cmsisdap::tools > Trying device Bus 020 Device 029: ID 1915:c00a
       DEBUG probe_rs::probe::cmsisdap::tools > Attempting to open 0d28:0204 in CMSIS-DAP v1 mode
       DEBUG probe_rs::probe::cmsisdap::commands > Draining probe of any pending data.
       DEBUG probe_rs::probe::cmsisdap::commands > Attempt 1 to find packet size
       TRACE probe_rs::probe::cmsisdap::commands > Transmit buffer: [00, 00, FF, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Receive buffer: [00, 02, 40, 00]...
       DEBUG probe_rs::probe::cmsisdap::commands > Success: packet size is 64
       DEBUG probe_rs::probe::cmsisdap::commands > Configuring probe to use packet size 64
       TRACE probe_rs::probe::cmsisdap::commands > Transmit buffer: [00, 00, FE, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Receive buffer: [00, 01, 04, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Transmit buffer: [00, 00, F0, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Receive buffer: [00, 01, 31, 00]...
       DEBUG probe_rs::probe::cmsisdap           > Detected probe capabilities: Capabilities { _swd_implemented: true, _jtag_implemented: false, swo_uart_implemented: false, swo_manchester_implemented: false, _atomic_commands_implemented: true, _test_domain_timer_implemented: true, swo_streaming_trace_implemented: false, _uart_communication_port_implemented: false, uart_com_port_implemented: false }
        INFO cargo_flash                         > Protocol speed 1000 kHz
        WARN probe_rs::session                   > XXXA
       DEBUG probe_rs::probe::cmsisdap           > Attaching to target system (clock = 1000kHz)
       TRACE probe_rs::probe::cmsisdap::commands > Transmit buffer: [00, 02, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Receive buffer: [02, 01, 00]...
        INFO probe_rs::probe::cmsisdap           > Using protocol SWD
       TRACE probe_rs::probe::cmsisdap::commands > Transmit buffer: [00, 11, 40, 42, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Receive buffer: [11, 00, 42, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Transmit buffer: [00, 04, 00, FF, FF, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Receive buffer: [04, 00, FF, FF, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Transmit buffer: [00, 13, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Receive buffer: [13, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Transmit buffer: [00, 01, 00, 01, 00]...
       TRACE probe_rs::probe::cmsisdap::commands > Receive buffer: [01, 00, 01, 00]...
        WARN probe_rs::architecture::arm::sequences::atsam > atsaml10 reset_hardware_assert
        WARN probe_rs::architecture::arm::sequences::atsam > atsaml10 do_cold_plug()
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
       TRACE probe_rs::probe::cmsisdap::commands           > Transmit buffer: [00, 10, 80, 80, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Receive buffer: [10, 83, 80, 00]...
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 131
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 0, select: 128, wait: 0
       TRACE probe_rs::probe::cmsisdap::commands           > Transmit buffer: [00, 10, 00, 80, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Receive buffer: [10, 03, 80, 00]...
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 3
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 0, select: 129, wait: 0
       TRACE probe_rs::probe::cmsisdap::commands           > Transmit buffer: [00, 10, 00, 81, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Receive buffer: [10, 02, 81, 00]...
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 2
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
       TRACE probe_rs::probe::cmsisdap::commands           > Transmit buffer: [00, 10, 80, 80, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Receive buffer: [10, 82, 80, 00]...
        WARN probe_rs::probe::cmsisdap                     > XXX cmsisdap:swj_pins response: 130
        WARN probe_rs::session                             > XXXB
        WARN probe_rs::session                             > XXXC
        WARN probe_rs::architecture::arm::sequences        > debug_port_setup
       TRACE probe_rs::probe::cmsisdap::commands           > Transmit buffer: [00, 12, 33, FF, FF, FF, FF, FF, FF, 07, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Receive buffer: [12, 00, FF, FF, FF, FF, FF, FF, 07, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Transmit buffer: [00, 12, 10, 9E, E7, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Receive buffer: [12, 00, 9E, E7, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Transmit buffer: [00, 12, 33, FF, FF, FF, FF, FF, FF, 07, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Receive buffer: [12, 00, FF, FF, FF, FF, FF, FF, 07, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Transmit buffer: [00, 12, 03, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Receive buffer: [12, 00]...
       DEBUG probe_rs::probe::cmsisdap                     > Adding command to batch: Read(port=DebugPort, addr=0)
       DEBUG probe_rs::probe::cmsisdap                     > 1 items in batch
       DEBUG probe_rs::probe::cmsisdap                     > Attempting batch of 1 items
       TRACE probe_rs::probe::cmsisdap::commands           > Transmit buffer: [00, 05, 00, 01, 02, 00]...
       TRACE probe_rs::probe::cmsisdap::commands           > Receive buffer: [05, 01, 01, 77, 14, F1, 0B, 00]...
       DEBUG probe_rs::probe::cmsisdap                     > 1 of batch of 1 items suceeded
       TRACE probe_rs::probe::cmsisdap                     > Transfer status: ACK
        WARN probe_rs::session                             > XXXD
       DEBUG probe_rs::architecture::arm::communication_interface > Selecting DP Default
        WARN probe_rs::architecture::arm::sequences               > debug_port_start
       DEBUG probe_rs::architecture::arm::dp                      > Writing DP register ABORT, value=0x0000001e
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=DebugPort, addr=0, data=0x0000001e
       DEBUG probe_rs::architecture::arm::dp                      > Writing DP register SELECT, value=0x00000000
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=DebugPort, addr=8, data=0x00000000
       DEBUG probe_rs::architecture::arm::dp                      > Reading DP register CTRL/STAT
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=DebugPort, addr=4)
       DEBUG probe_rs::probe::cmsisdap                            > 3 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 3 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 03, 00, 1E, 00, 00, 00, 08, 00, 00, 00, 00, 06, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 03, 01, 00, 00, 00, 00, 00, 08, 00, 00, 00, 00, 06, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 3 of batch of 3 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::dp                      > Read    DP register CTRL/STAT, value=0x00000000
       DEBUG probe_rs::architecture::arm::dp                      > Writing DP register CTRL/STAT, value=0x50000000
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=DebugPort, addr=4, data=0x50000000
       DEBUG probe_rs::architecture::arm::dp                      > Reading DP register CTRL/STAT
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=DebugPort, addr=4)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 04, 00, 00, 00, 50, 06, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 00, 00, F0, 50, 06, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::dp                      > Read    DP register CTRL/STAT, value=0xf0000000
       DEBUG probe_rs::architecture::arm::dp                      > Writing DP register CTRL/STAT, value=0x50000f00
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=DebugPort, addr=4, data=0x50000f00
       DEBUG probe_rs::architecture::arm::dp                      > Reading DP register CTRL/STAT
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=DebugPort, addr=4)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 04, 00, 0F, 00, 50, 06, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 00, 00, F0, 50, 06, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::dp                      > Read    DP register CTRL/STAT, value=0xf0000000
       DEBUG probe_rs::architecture::arm::dp                      > Reading DP register CTRL/STAT
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=DebugPort, addr=4)
       DEBUG probe_rs::probe::cmsisdap                            > 1 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 1 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 01, 06, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 01, 01, 00, 00, 00, F0, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 1 of batch of 1 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::dp                      > Read    DP register CTRL/STAT, value=0xf0000000
       DEBUG probe_rs::architecture::arm::dp                      > Writing DP register CTRL/STAT, value=0xf0000000
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=DebugPort, addr=4, data=0xf0000000
       TRACE probe_rs::architecture::arm::communication_interface > Searching valid APs
       DEBUG probe_rs::architecture::arm::ap                      > Reading register IDR
       DEBUG probe_rs::architecture::arm::communication_interface > Changing AP to 0, AP_BANK_SEL to 15
       DEBUG probe_rs::architecture::arm::dp                      > Writing DP register SELECT, value=0x000000f0
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=DebugPort, addr=8, data=0x000000f0
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=252)
       DEBUG probe_rs::probe::cmsisdap                            > 3 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 3 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 03, 04, 00, 00, 00, F0, 08, F0, 00, 00, 00, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 03, 01, 25, 00, 77, 04, F0, 08, F0, 00, 00, 00, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 3 of batch of 3 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    IDR, value=0x4770025
       DEBUG probe_rs::architecture::arm::ap                      > Reading register IDR
       DEBUG probe_rs::architecture::arm::communication_interface > Changing AP to 1, AP_BANK_SEL to 15
       DEBUG probe_rs::architecture::arm::dp                      > Writing DP register SELECT, value=0x010000f0
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=DebugPort, addr=8, data=0x010000f0
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=252)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 08, F0, 00, 00, 01, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 00, 00, 00, 01, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    IDR, value=0x0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register IDR
       DEBUG probe_rs::architecture::arm::communication_interface > Changing AP to 0, AP_BANK_SEL to 15
       DEBUG probe_rs::architecture::arm::dp                      > Writing DP register SELECT, value=0x000000f0
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=DebugPort, addr=8, data=0x000000f0
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=252)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 08, F0, 00, 00, 00, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 25, 00, 77, 04, 00, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    IDR, value=0x4770025
       DEBUG probe_rs::architecture::arm::ap                      > Reading register BASE
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=248)
       DEBUG probe_rs::probe::cmsisdap                            > 1 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 1 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 01, 0B, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 01, 01, 03, 30, 00, 41, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 1 of batch of 1 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    BASE, value=0x41003003
       DEBUG probe_rs::architecture::arm::ap                      > Reading register BASE2
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=240)
       DEBUG probe_rs::probe::cmsisdap                            > 1 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 1 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 01, 03, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 01, 01, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 1 of batch of 1 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    BASE2, value=0x0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register CSW
       DEBUG probe_rs::architecture::arm::communication_interface > Changing AP to 0, AP_BANK_SEL to 0
       DEBUG probe_rs::architecture::arm::dp                      > Writing DP register SELECT, value=0x00000000
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=DebugPort, addr=8, data=0x00000000
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=0)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 08, 00, 00, 00, 00, 03, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 40, 00, 00, 43, 00, 03, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    CSW, value=0x43000040
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 1, HNONSEC: 1, PROT: 6, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U8 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0xe3000010
       DEBUG probe_rs::architecture::arm::ap                      > Reading register CSW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=0)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 01, 10, 00, 00, E3, 03, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 50, 00, 00, 43, E3, 03, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    CSW, value=0x43000050
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 1, PROT: 0, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 1, AddrInc: Off, _RES1: 0, SIZE: U8 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0x43000040
       DEBUG probe_rs::architecture::arm::communication_interface > HNONSEC supported: true
       DEBUG probe_rs::architecture::arm::ap                      > Reading register CFG
       DEBUG probe_rs::architecture::arm::communication_interface > Changing AP to 0, AP_BANK_SEL to 15
       DEBUG probe_rs::architecture::arm::dp                      > Writing DP register SELECT, value=0x000000f0
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=DebugPort, addr=8, data=0x000000f0
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=244)
       DEBUG probe_rs::probe::cmsisdap                            > 3 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 3 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 03, 01, 40, 00, 00, 43, 08, F0, 00, 00, 00, 07, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 03, 01, 00, 00, 00, 00, 43, 08, F0, 00, 00, 00, 07, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 3 of batch of 3 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    CFG, value=0x0
       DEBUG probe_rs::architecture::arm::communication_interface > AP GenericAp { address: ApAddress { dp: Default, ap: 0 } }: MemoryAp(MemoryApInformation { address: ApAddress { dp: Default, ap: 0 }, only_32bit_data_size: false, debug_base_address: 1090531328, supports_hnonsec: true, has_large_address_extension: false, has_large_data_extension: false })
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U8 }
       DEBUG probe_rs::architecture::arm::communication_interface > Changing AP to 0, AP_BANK_SEL to 0
       DEBUG probe_rs::architecture::arm::dp                      > Writing DP register SELECT, value=0x00000000
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=DebugPort, addr=8, data=0x00000000
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000010
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002101 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002101
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 4 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 4 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 04, 08, 00, 00, 00, 00, 01, 10, 00, 00, 23, 05, 01, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 04, 01, 00, 22, 00, 18, 00, 01, 10, 00, 00, 23, 05, 01, 21, 00, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 4 of batch of 4 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x18002200
        WARN probe_rs::architecture::arm::sequences::atsam        > XXXa1
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002101 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002101
       DEBUG probe_rs::architecture::arm::ap                      > Writing register DRW, value=DRW { data: 200 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=12, data=0x00000200
        WARN probe_rs::architecture::arm::sequences::atsam        > XXXa2
        WARN probe_rs::architecture::arm::sequences::atsam        > XXXa3
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 4 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 4 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 04, 05, 01, 21, 00, 41, 0D, 00, 02, 00, 00, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 04, 01, 00, 20, 04, 18, 41, 0D, 00, 02, 00, 00, 05, 02, 21, 00, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 4 of batch of 4 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x18042000
        WARN probe_rs::architecture::arm::sequences::atsam        > XXXa4
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002120 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002120
       DEBUG probe_rs::architecture::arm::ap                      > Writing register DRW, value=DRW { data: 444247aa }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=12, data=0x444247aa
        WARN probe_rs::architecture::arm::sequences::atsam        > XXXaa
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U8 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000010
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 6 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 6 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 06, 01, 12, 00, 00, 23, 05, 20, 21, 00, 41, 0D, AA, 47, 42, 44, 01, 10, 00, 00, 23, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 06, 01, 00, 20, 04, 18, 23, 05, 20, 21, 00, 41, 0D, AA, 47, 42, 44, 01, 10, 00, 00, 23, 05, 02, 21, 00, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 6 of batch of 6 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x18042000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 8E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x188e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002124 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002124
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 3 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 3 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 03, 01, 12, 00, 00, 23, 05, 24, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 03, 01, 39, 00, 00, EC, 23, 05, 24, 21, 00, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 3 of batch of 3 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0xec000039
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U8 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000010
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 3 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 3 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 03, 01, 10, 00, 00, 23, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 03, 01, 00, 20, 0E, 18, 23, 05, 02, 21, 00, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 3 of batch of 3 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: 41002102 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0x41002102
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 02, 21, 00, 41, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 00, 20, 0E, 18, 41, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x180e2000
        WARN probe_rs::session                                    > XXXE
        WARN probe_rs::architecture::arm::sequences               > debug_core_start
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 3 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 3 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 03, 01, 12, 00, 00, 23, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 03, 01, 00, 00, 10, 03, 23, 05, F0, ED, 00, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 3 of batch of 3 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x3100000
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Writing register DRW, value=DRW { data: a05f0001 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=12, data=0xa05f0001
        WARN probe_rs::session                                    > XXXF
        WARN probe_rs::architecture::arm::sequences               > reset_catch_set
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edfc }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edfc
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 5 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 5 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 05, 05, F0, ED, 00, E0, 0D, 01, 00, 5F, A0, 01, 12, 00, 00, 23, 05, FC, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 05, 01, 00, 00, 00, 00, E0, 0D, 01, 00, 5F, A0, 01, 12, 00, 00, 23, 05, FC, ED, 00, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 5 of batch of 5 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x0
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edfc }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edfc
       DEBUG probe_rs::architecture::arm::ap                      > Writing register DRW, value=DRW { data: 1 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=12, data=0x00000001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 4 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 4 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 04, 05, FC, ED, 00, E0, 0D, 01, 00, 00, 00, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 04, 01, 01, 00, 10, 01, E0, 0D, 01, 00, 00, 00, 05, F0, ED, 00, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 4 of batch of 4 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
        WARN probe_rs::architecture::arm::sequences               > reset_hardware_deassert
        WARN probe_rs::probe::cmsisdap                            > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 10, 80, 80, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [10, 83, 80, 00]...
        WARN probe_rs::probe::cmsisdap                            > XXX cmsisdap:swj_pins response: 131
        WARN probe_rs::probe::cmsisdap                            > XXX cmsisdap:swj_pins request out: 128, select: 128, wait: 0
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 10, 80, 80, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [10, 83, 80, 00]...
        WARN probe_rs::probe::cmsisdap                            > XXX cmsisdap:swj_pins response: 131
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 3 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 3 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 03, 01, 12, 00, 00, 23, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 03, 01, 01, 00, 10, 01, 23, 05, F0, ED, 00, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 3 of batch of 3 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::core::armv8m            > State when connecting: Dhcsr { .0: 1100001, s_restart_st: false, s_reset_st: false, s_retire_st: true, s_fpd: false, s_suide: false, s_nsuide: false, s_sde: true, s_lockup: false, s_sleep: false, s_halt: false, s_regrdy: false, c_pmov: false, c_snapstall: false, c_maskints: false, c_step: false, c_halt: false, c_debugen: true }
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000ed30 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000ed30
       DEBUG probe_rs::architecture::arm::ap                      > Writing register DRW, value=DRW { data: 1f }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=12, data=0x0000001f
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 4 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 4 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 04, 05, 30, ED, 00, E0, 0D, 1F, 00, 00, 00, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 04, 01, 01, 00, 10, 01, E0, 0D, 1F, 00, 00, 00, 05, F0, ED, 00, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 4 of batch of 4 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edf0 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edf0
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, F0, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 01, 00, 10, 01, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1100001
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e0002000 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe0002000
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 3 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 3 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 03, 01, 12, 00, 00, 23, 05, 00, 20, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 03, 01, 40, 00, 00, 10, 23, 05, 00, 20, 00, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 3 of batch of 3 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x10000040
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e0002008 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe0002008
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 08, 20, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, FE, FF, FB, FF, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0xfffbfffe
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000200c }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000200c
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 0C, 20, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 7E, FD, FF, FB, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0xfbfffd7e
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e0002010 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe0002010
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 10, 20, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, FE, FE, FF, FF, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0xfffffefe
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e0002014 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe0002014
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, 14, 20, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, FE, BF, FF, FF, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0xffffbffe
       DEBUG probe_rs::architecture::arm::ap                      > Writing register CSW, value=CSW { DbgSwEnable: 0, HNONSEC: 0, PROT: 2, CACHE: 3, SPIDEN: 0, _RES0: 0, MTE: 0, Type: 0, Mode: 0, TrinProg: 0, DeviceEn: 0, AddrInc: Single, _RES1: 0, SIZE: U32 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=0, data=0x23000012
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edfc }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edfc
       DEBUG probe_rs::architecture::arm::ap                      > Reading register DRW
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Read(port=AccessPort, addr=12)
       DEBUG probe_rs::probe::cmsisdap                            > 3 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 3 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 03, 01, 12, 00, 00, 23, 05, FC, ED, 00, E0, 0F, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 03, 01, 01, 00, 00, 00, 23, 05, FC, ED, 00, E0, 0F, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 3 of batch of 3 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       DEBUG probe_rs::architecture::arm::ap                      > Read register    DRW, value=0x1
       DEBUG probe_rs::architecture::arm::ap                      > Writing register TAR, value=TAR { address: e000edfc }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=4, data=0xe000edfc
       DEBUG probe_rs::architecture::arm::ap                      > Writing register DRW, value=DRW { data: 1 }
       DEBUG probe_rs::probe::cmsisdap                            > Adding command to batch: Write(port=AccessPort, addr=12, data=0x00000001
        WARN probe_rs::architecture::arm::sequences               > debug_core_stop
       DEBUG probe_rs::probe::cmsisdap                            > Detaching from CMSIS-DAP probe
       DEBUG probe_rs::probe::cmsisdap                            > 2 items in batch
       DEBUG probe_rs::probe::cmsisdap                            > Attempting batch of 2 items
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 05, 00, 02, 05, FC, ED, 00, E0, 0D, 01, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [05, 02, 01, 05, FC, ED, 00, E0, 0D, 01, 00]...
       DEBUG probe_rs::probe::cmsisdap                            > 2 of batch of 2 items suceeded
       TRACE probe_rs::probe::cmsisdap                            > Transfer status: ACK
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 03, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [03, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Transmit buffer: [00, 01, 00]...
       TRACE probe_rs::probe::cmsisdap::commands                  > Receive buffer: [01, 00]...
        INFO cargo_flash                                          > Metadata {
    chip: Some(
        "\"ATSAML10E16A\"",
    ),
    probe: Some(
        "\"CMSIS-DAP\"",
    ),
    speed: Some(
        "1000",
    ),
    release: "0.12.1",
    commit: "v0.12.1-34-g2aedd62-modified",
}
       Error Connecting to the chip was unsuccessful.

  Caused by:
          0: An error with the usage of the probe occurred
          1: Operation timed out
```

## 26 June 2022

Ok, I'm pretty sure the flash code is dying in the attach under reset
code path in the XXXF section. Which is weird because we already did
ann attach under reset up above in XXXA...

Ok, added some new debug prints:

XXXF2 - `reset_hardware_deassert` is messing with the pins and probably
messing things up.

XXXF3 appears to be the thing that's timing out.


---

Ok, current code doesn't work because I'm reseting the chip without
re-connecting (code in debug_port_setup).
