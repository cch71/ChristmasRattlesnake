use embedded_hal::digital::OutputPin;

///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
pub(super) struct RattleControl<O>
where
    O: OutputPin,
{
    freq: i8,
    delay: i8,
    is_lit: bool,
    ssr_channel: O,
}

///////////////////////////////////////////////////////////////////////////
impl<O> RattleControl<O>
where
    O: OutputPin,
{
    ///////////////////////////////////////////////////////////////////////////
    pub(super) fn new(mut ssr_channel_pin: O) -> Self {
        ssr_channel_pin.set_high().unwrap();

        Self {
            freq: -1,
            delay: 0,
            is_lit: true,
            ssr_channel: ssr_channel_pin,
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    pub(super) fn set_distance_as_inches(&mut self, inches_to_target: Option<u8>) {
        // Calculate the frequency we should be oscillating the christmas
        // lights
        let current_freq = match inches_to_target {
            Some(0..=5) => 0,
            Some(6..=15) => 1,
            Some(16..=36) => 5,
            _ => -1,
        };

        // If they are 0 or -1 then we can just peg it and not have to
        // set it every time.
        if current_freq != self.freq {
            self.freq = current_freq;
            if self.freq == -1 || self.freq == 0 {
                self.ssr_channel.set_high().unwrap();
            }

            // Reset the delay so it starts fresh
            self.delay = 0;
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    pub(super) fn next_maybe_rattle(&mut self) {
        // Based on the count of times being called flip the lights
        // but only of it is not 0 or -1 which are solid vals
        if self.freq > 0 && self.delay != self.freq {
            self.delay += 1;

            if self.delay == self.freq {
                match self.is_lit {
                    true => self.ssr_channel.set_high(),
                    _ => self.ssr_channel.set_low(),
                }
                .unwrap();
                self.is_lit = !self.is_lit;
                self.delay = 0;
            }
        }
    }
}
