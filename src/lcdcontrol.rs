
use crate::lcd_lm1602_i2c_driver::Lcd;

use embedded_hal::blocking::{
    delay::DelayMs,
    i2c::Write,
};
use esp_println::println;

const LCD_ADDRESS: u8 = 0x27;

///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
///
pub(super) struct LcdControl
{
    lcd: Lcd,
}
////////////////////////////////////////////////////////////////////////////
///
impl LcdControl
{
    ////////////////////////////////////////////////////////////////////////////
    ///
    pub(super) fn new<I: Write, D: DelayMs<u8>>(i2c: &mut I, delay: &mut D) -> Result<Self, <I as Write>::Error> {
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

        Ok(Self {
            lcd,
        })
    }

    ////////////////////////////////////////////////////////////////////////////
    ///
    pub(super) fn update_distance<I: Write, D: DelayMs<u8>>(&mut self, i2c: &mut I, delay: &mut D, inches_to_target: u8) -> Result<(), <I as Write>::Error> {
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
    pub(super) fn clear_distance<I: Write, D: DelayMs<u8>>(&mut self, i2c: &mut I, delay: &mut D) -> Result<(), <I as Write>::Error> {
        self.lcd.set_cursor(i2c, delay, 1, 5)?;
        self.lcd.write_str(i2c, delay, "     ")?;
        Ok(())
    }

}
