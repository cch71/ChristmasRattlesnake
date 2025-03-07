use crate::lcd_lm1602_i2c_driver::Lcd;

use embedded_hal::{delay::DelayNs, i2c::I2c};
use esp_println::println;

const LCD_ADDRESS: u8 = 0x27;

///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
///
pub(super) struct LcdControl {
    lcd: Lcd,
}
////////////////////////////////////////////////////////////////////////////
///
impl LcdControl {
    ////////////////////////////////////////////////////////////////////////////
    ///
    pub(super) fn new<I2C: I2c, D: DelayNs>(
        i2c: &mut I2C,
        delay: &mut D,
    ) -> Result<Self, I2C::Error> {
        println!("Setting LCD\r");
        let mut lcd = Lcd::new()
            .address(LCD_ADDRESS)
            .cursor_on(false) // no visible cursor
            .rows(2) // two rows
            .init(i2c, delay)?;

        lcd.clear(i2c, delay)?;
        lcd.return_home(i2c, delay)?;
        lcd.write_str(i2c, delay, "XMas Rattler")?;
        lcd.set_cursor(i2c, delay, 1, 0)?;
        lcd.write_str(i2c, delay, "Dist:")?;
        println!("Done Setting up LCD\r");

        Ok(Self { lcd })
    }

    ////////////////////////////////////////////////////////////////////////////
    ///
    pub(super) fn update_distance<I2C: I2c, D: DelayNs>(
        &mut self,
        i2c: &mut I2C,
        delay: &mut D,
        inches_to_target: u8,
    ) -> Result<(), I2C::Error> {
        self.lcd.set_cursor(i2c, delay, 1, 5)?;

        use core::fmt::Write;
        use heapless::String;
        let mut data = String::<5>::new();
        let _ = write!(data, "{:03}in", inches_to_target);

        self.lcd.write_str(i2c, delay, data.as_str())?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////
    ///
    pub(super) fn clear_distance<I2C: I2c, D: DelayNs>(
        &mut self,
        i2c: &mut I2C,
        delay: &mut D,
    ) -> Result<(), I2C::Error> {
        self.lcd.set_cursor(i2c, delay, 1, 5)?;
        self.lcd.write_str(i2c, delay, "     ")?;
        Ok(())
    }
}
