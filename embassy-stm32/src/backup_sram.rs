//! Battary backed SRAM

use core::mem;

use embassy_hal_internal::Peri;

use crate::_generated::{BKPSRAM_BASE, BKPSRAM_SIZE};
use crate::peripherals::BKPSRAM;

/// Struct used to initilize backup sram
pub struct BackupMemory {
    // true if the sram was retained across last reset
    retained: bool,
}

impl BackupMemory {
    /// Setup battery backed sram
    ///
    /// Returns slice to sram and whether the sram was retained
    pub fn new(_backup_sram: Peri<'static, BKPSRAM>) -> Self {
        // Assert bksram has been enabled in rcc
        assert!(crate::pac::PWR.bdcr().read().bren() == crate::pac::pwr::vals::Retention::Preserved);

        Self {
            // SAFETY: It is safe to read this static mut in the CS
            retained: critical_section::with(|_| unsafe { crate::rcc::BKSRAM_RETAINED }),
        }
    }

    /// Returns true if the sram was retained across last reset
    pub fn is_retained(&self) -> bool {
        self.retained
    }

    /// Get raw pointer to the battery backed memory
    ///
    /// Note that this is not necesserily normal memory, so please do use volatile
    /// and aligned reads/writes unless you know what you are doing.
    pub fn as_ptr(&self) -> *mut u8 {
        BKPSRAM_BASE as *mut u8
    }

    /// Size of backup sram
    pub fn size(&self) -> usize {
        BKPSRAM_SIZE
    }

    /// Write single byte to backup sram
    ///
    /// Address is relative start of backup sram
    pub fn read(&mut self, address: usize, dst: &mut [u8]) {
        assert!(address + dst.len() <= self.size());
        let p = unsafe { self.as_ptr().add(address) };

        let (prefix, middle, suffix) = unsafe { dst.align_to_mut::<usize>() };
        let mut i = 0;

        for b in prefix {
            // SAFETY: Single byte reads are safe to perform into the backup sram
            unsafe {
                *b = p.add(i).read_volatile();
            }
            i += i;
        }
        for x in middle {
            // SAFETY: Word sized reads are safe to perform into the backup sram since they are aligned
            unsafe {
                *x = p.add(i).cast::<usize>().read_volatile();
            }
            i += mem::size_of::<usize>();
        }
        for b in suffix {
            // SAFETY: Single byte reads are safe to perform into the backup sram
            unsafe {
                *b = p.add(i).read_volatile();
            }
            i += i;
        }
    }

    /// Write single byte to backup sram
    ///
    /// Address is relative start of backup sram
    pub fn write(&mut self, address: usize, src: &[u8]) {
        assert!(address + src.len() <= self.size());
        let p = unsafe { self.as_ptr().add(address) };

        let (prefix, middle, suffix) = unsafe { src.align_to::<usize>() };
        let mut i = 0;

        for &b in prefix {
            // SAFETY: Single byte writes are safe to perform into the backup sram
            unsafe {
                p.add(i).write_volatile(b);
            }
            i += i;
        }
        for &x in middle {
            // SAFETY: Word sized writes are safe to perform into the backup sram since they are aligned
            unsafe {
                p.add(i).cast::<usize>().write_volatile(x);
            }
            i += mem::size_of::<usize>();
        }
        for &b in suffix {
            // SAFETY: Single byte writes are safe to perform into the backup sram
            unsafe {
                p.add(i).write_volatile(b);
            }
            i += i;
        }
    }
}
