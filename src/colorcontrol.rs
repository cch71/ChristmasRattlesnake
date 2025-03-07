use esp_hal::rmt::TxChannel;
use esp_hal_smartled::SmartLedsAdapter;
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB,
};

///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
pub(super) struct ColorControl<TX, const BUFFER_SIZE: usize>
where
    TX: TxChannel,
{
    led_controller: SmartLedsAdapter<TX, BUFFER_SIZE>,
    data: [RGB<u8>; 1],
    current_color: Hsv,
}

impl<TX, const BUFFER_SIZE: usize> ColorControl<TX, BUFFER_SIZE>
where
    TX: TxChannel,
{
    ///////////////////////////////////////////////////////////////////////////
    /// Create a new adapter object that drives the pin using the RMT channel.
    pub(super) fn new(mut led_controller: SmartLedsAdapter<TX, BUFFER_SIZE>) -> Self {
        // Initialize color to green
        let mut color = Hsv {
            hue: 64,
            sat: 255,
            val: 255,
        };

        let data = [hsv2rgb(color)];
        led_controller
            .write(brightness(gamma(data.iter().cloned()), 10))
            .unwrap();

        color.hue = 0;
        Self {
            led_controller,
            data,
            current_color: color,
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    /// for hue in 0..=255 {
    pub(super) fn next(&mut self) {
        // Convert from the HSV color space (where we can easily transition from one
        // color to the other) to the RGB color space that we can then send to the LED
        self.data = [hsv2rgb(self.current_color)];
        // When sending to the LED, we do a gamma correction first (see smart_leds
        // documentation for details) and then limit the brightness to 10 out of 255 so
        // that the output it's not too bright.
        self.led_controller
            .write(brightness(gamma(self.data.iter().cloned()), 10))
            .unwrap();
        // Rotate colors
        if self.current_color.hue == 255 {
            self.current_color.hue = 0;
        } else {
            self.current_color.hue += 1;
        }
    }
}
