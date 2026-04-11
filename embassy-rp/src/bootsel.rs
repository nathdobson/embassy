//! Boot Select button
//!
//! The RP2040 rom supports a BOOTSEL button that is used to enter the USB bootloader
//! if held during reset. To avoid wasting GPIO pins, the button is multiplexed onto
//! the CS pin of the QSPI flash, but that makes it somewhat expensive and complicated
//! to utilize outside of the rom's bootloader.
//!
//! This module provides functionality to poll BOOTSEL from an embassy application.

use crate::Peri;
use crate::flash::in_ram;

/// Reads the BOOTSEL button. Returns true if the button is pressed.
///
/// Reading isn't cheap, as this function waits for core 1 to finish it's current
/// task and for any DMAs from flash to complete
pub fn is_bootsel_pressed(_p: Peri<'_, crate::peripherals::BOOTSEL>) -> bool {
    let mut cs_status = 0u32;

    unsafe { in_ram(|| cs_status = ram_helpers::read_cs_status()) }
        .expect("Must be called from Core 0");
    info!("{}", cs_status);
    cfg_if::cfg_if! {
        if #[cfg(feature = "rp2040")] {
            cs_status & 1 != 0
        }else if #[cfg(feature = "_rp235x")] {
            cs_status & 0x08000000 != 0
        }else{
            compile_error!("chip not supported")
        }
    }
}

mod ram_helpers {
    use rp_pac::IO_QSPI;
    use rp_pac::SIO;

    /// Temporally reconfigures the CS gpio and returns the GpioStatus.

    /// This function runs from RAM so it can disable flash XIP.
    ///
    /// # Safety
    ///
    /// The caller must ensure flash is idle and will remain idle.
    /// This function must live in ram. It uses inline asm to avoid any
    /// potential calls to ABI functions that might be in flash.
    #[inline(never)]
    #[unsafe(link_section = ".data.ram_func")]
    #[cfg(target_arch = "arm")]
    pub unsafe fn read_cs_status() -> u32 {
        let mut result: u32 = 0;

        let delay = 1000;
        const REG_ALIAS_XOR_BITS: usize = 1 << 12;

        core::arch::asm!(
            ".equiv GPIO_REG, 0x1c",
            ".equiv READ_REG, 0x08",

            // The BOOTSEL pulls the flash's CS line low though a 1K resistor.
            // this is weak enough to avoid disrupting normal operation.
            // But, if we disable CS's output drive and allow it to float...
            // "str {val}, [{cs_gpio2}, $GPIO_CTRL]",
            "ldr {ctrl}, [{gpio}, $GPIO_REG]",
            "eor {ctrl}, {ctrl}, #0x8000",
            "and {ctrl}, {ctrl}, #0xc000",
            "str {ctrl}, [{gpio_xor}, $GPIO_REG]",

            // ...then wait for the state to settle...
            "2:", // ~4000 cycle delay loop
            "subs {val}, #1",
            "bne 2b",

            // ...we can read the current state of bootsel
            "ldr {val}, [{gpio_read}, $READ_REG]",

            // Finally, restore CS to normal operation so XIP can continue
            "ldr {ctrl}, [{gpio}, $GPIO_REG]",
            "and {ctrl}, {ctrl}, #0xc000",
            "str {ctrl}, [{gpio_xor}, $GPIO_REG]",

            gpio = in(reg) IO_QSPI.as_ptr(),
            gpio_xor = in(reg) IO_QSPI.as_ptr() as usize + REG_ALIAS_XOR_BITS,
            gpio_read = in(reg) SIO.as_ptr(),
            ctrl = out(reg) _,
            val = inout(reg) delay => result,
            options(nostack),
        );

        result
    }

    #[cfg(not(target_arch = "arm"))]
    pub unsafe fn read_cs_status() -> GpioStatus {
        unimplemented!()
    }
}
