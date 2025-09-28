use x86_64::instructions::port::Port;
use embedded_time::duration::{Hours, Minutes, Seconds};

/// RTC driver for CMOS RTC
pub struct Rtc;

impl Rtc {
    /// Read a CMOS register
    fn read_register(reg: u8) -> u8 {
        unsafe {
            let mut addr_port = Port::new(0x70);
            let mut data_port = Port::new(0x71);
            
            addr_port.write(reg);
            data_port.read()
        }
    }

    /// Wait until RTC is not updating
    fn wait_not_updating() {
        while Self::read_register(0x0A) & 0x80 != 0 {}
    }

    /// Convert BCD to binary if needed
    fn bcd_to_bin(val: u8) -> u8 {
        (val & 0x0F) + ((val / 16) * 10)
    }
    
    /// Read time (hour, min, sec)
    pub fn read_time() -> (Hours, Minutes, Seconds) {
        Self::wait_not_updating();
        
        let mut sec = Self::read_register(0x00);
        let mut min = Self::read_register(0x02);
        let mut hour = Self::read_register(0x04);
        
        let status_b = Self::read_register(0x0B);
        if status_b & 0x04 == 0 {
            sec = Self::bcd_to_bin(sec);
            min = Self::bcd_to_bin(min);
            hour = Self::bcd_to_bin(hour);
        }

        (Hours::new(hour as u32), Minutes::new(min as u32), Seconds::new(sec as u32))
    }
    
    /// Read date (day, month, year)
    pub fn read_date() -> (u8, u8, u16) {
        Self::wait_not_updating();
        
        let mut day = Self::read_register(0x07);
        let mut month = Self::read_register(0x08);
        let mut year = Self::read_register(0x09) as u16;
        let mut cent = Self::read_register(0x32);

        // check BCD mode
        let status_b = Self::read_register(0x0B);
        if status_b & 0x04 == 0 {
            day = Self::bcd_to_bin(day);
            month = Self::bcd_to_bin(month);
            year = Self::bcd_to_bin(year as u8) as u16;
            cent = Self::bcd_to_bin(cent)
        }

        year += cent as u16 * 100;

        (day, month, year)
    }
}
