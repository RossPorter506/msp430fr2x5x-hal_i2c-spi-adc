use crate::gpio::*;
use core::convert::Infallible;
use embedded_hal::adc::{Channel, OneShot};
use msp430fr2355::ADC;

pub enum SampleTime {
    _4,
    _8,
    _16,
    _32,
    _64,
    _96,
    _128,
    _192,
    _256,
    _384,
    _512,
    _768,
    _1024,
}

impl SampleTime {
    fn adcsht(self) -> u8 {
        match self {
            SampleTime::_4 => 0b000,
            SampleTime::_8 => 0b001,
            SampleTime::_16 => 0b010,
            SampleTime::_32 => 0b011,
            SampleTime::_64 => 0b100,
            SampleTime::_96 => 0b101,
            SampleTime::_128 => 0b110,
            SampleTime::_192 => 0b111,
            SampleTime::_256 => 0b1000,
            SampleTime::_384 => 0b1001,
            SampleTime::_512 => 0b1010,
            SampleTime::_768 => 0b1011,
            SampleTime::_1024 => 0b1100,
        }
    }
}

pub enum ClockDivider {
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
}

impl ClockDivider {
    fn adcdiv(self) -> u8 {
        match self {
            ClockDivider::_1 => 0b000,
            ClockDivider::_2 => 0b001,
            ClockDivider::_3 => 0b010,
            ClockDivider::_4 => 0b011,
            ClockDivider::_5 => 0b100,
            ClockDivider::_6 => 0b101,
            ClockDivider::_7 => 0b110,
            ClockDivider::_8 => 0b111,
        }
    }
}

pub enum ClockSource {
    MODCLK,
    ACLK,
    SMCLK,
}

impl ClockSource {
    fn adcssel(self) -> u8 {
        match self {
            ClockSource::MODCLK => 0b00,
            ClockSource::ACLK => 0b01,
            ClockSource::SMCLK => 0b10,
        }
    }
}

pub enum Predivider {
    _1,
    _4,
    _64,
}

impl Predivider {
    fn adcpdiv(self) -> u8 {
        match self {
            Predivider::_1 => 0b00,
            Predivider::_4 => 0b01,
            Predivider::_64 => 0b10,
        }
    }
}

pub enum Resolution {
    _8BIT,
    _10BIT,
    _12BIT,
}

impl Resolution {
    fn adcres(self) -> u8 {
        match self {
            Resolution::_8BIT => 0b00,
            Resolution::_10BIT => 0b01,
            Resolution::_12BIT => 0b10,
        }
    }
}

pub enum SamplingRate {
    _50KSPS,
    _200KSPS,
}

impl SamplingRate {
    fn adcsr(self) -> bool {
        match self {
            SamplingRate::_200KSPS => false,
            SamplingRate::_50KSPS => true,
        }
    }
}

// Pins corresponding to an ADC channel. Pin types can have `::channel()` called on them to get their ADC channel index.
macro_rules! impl_adc_channel {
    ($port: ty, $pin: ty, $channel: literal ) => {
        impl Channel<Adc<ADC>> for Pin<$port, $pin, Alternate3<Input<Floating>>> {
            type ID = u8;

            fn channel() -> Self::ID {
                $channel
            }
        }
    };
}

impl_adc_channel!(P1, Pin0, 0);
impl_adc_channel!(P1, Pin1, 1);
impl_adc_channel!(P1, Pin2, 2);
impl_adc_channel!(P1, Pin3, 3);
impl_adc_channel!(P1, Pin4, 4);
impl_adc_channel!(P1, Pin5, 5);
impl_adc_channel!(P1, Pin6, 6);
impl_adc_channel!(P1, Pin7, 7);
impl_adc_channel!(P5, Pin0, 8);
impl_adc_channel!(P5, Pin1, 9);
impl_adc_channel!(P5, Pin2, 10);
impl_adc_channel!(P5, Pin3, 11);

pub struct Adc<ADC> {
    adc_reg: ADC,
    is_waiting: bool,
}

pub struct AdcConfig {
    pub adc: ADC,
    pub clock_source: ClockSource,
    pub clock_divider: ClockDivider,
    pub predivider: Predivider,
    pub resolution: Resolution,
    pub sampling_rate: SamplingRate,
    pub sample_time: SampleTime,
}

impl AdcConfig {
    /// Creates an ADC configuration
    pub fn new(
        adc: ADC,
        clock_source: ClockSource,
        clock_divider: ClockDivider,
        predivider: Predivider,
        resolution: Resolution,
        sampling_rate: SamplingRate,
        sample_time: SampleTime,
    ) -> AdcConfig {
        AdcConfig {
            adc,
            clock_source,
            clock_divider,
            predivider,
            resolution,
            sampling_rate,
            sample_time,
        }
    }

    /// Applies this ADC configuration to hardware registers, and returns an ADC.
    pub fn config_hw(self) -> Adc<ADC> {
        let adc_reg = self.adc;
        unsafe {
            adc_reg.adcctl0.clear_bits(|w| {
                w.adcenc()
                    .clear_bit()
                    .adcon()
                    .clear_bit()
                    .adcsc()
                    .clear_bit()
            });
        }
        let adcsht = self.sample_time.adcsht();
        adc_reg.adcctl0.modify(|_, w| w.adcsht().bits(adcsht));

        let adcssel = self.clock_source.adcssel();
        adc_reg
            .adcctl1
            .modify(|_, w| w.adcssel().bits(adcssel).adcshp().adcshp_1());

        let adcdiv = self.clock_divider.adcdiv();
        adc_reg.adcctl1.modify(|_, w| w.adcdiv().bits(adcdiv));

        let adcpdiv = self.predivider.adcpdiv();
        adc_reg.adcctl2.modify(|_, w| w.adcpdiv().bits(adcpdiv));

        let adcres = self.resolution.adcres();
        adc_reg.adcctl2.modify(|_, w| w.adcres().bits(adcres));

        let adcsr = self.sampling_rate.adcsr();
        adc_reg.adcctl2.modify(|_, w| w.adcsr().bit(adcsr));

        Adc {
            adc_reg,
            is_waiting: false,
        }
    }
}

impl Adc<ADC> {
    /// Create an ADC instance with a default configuration.
    ///
    /// If you need a custom configuration you should construct an ADC using AdcConfig instead.
    pub fn new(adc: ADC) -> Adc<ADC> {
        Adc {
            adc_reg: adc,
            is_waiting: false,
        }
    }

    /// Enables this ADC, ready to start a conversion.
    pub fn adc_enable(&mut self) {
        unsafe {
            self.adc_reg.adcctl0.set_bits(|w| w.adcon().set_bit());
        }
    }

    /// Disables this ADC to save power.
    pub fn adc_disable(&mut self) {
        unsafe {
            self.adc_reg
                .adcctl0
                .clear_bits(|w| w.adcon().clear_bit().adcenc().clear_bit());
        }
    }

    /// Starts an ADC conversion.
    pub fn adc_start_conversion(&mut self) {
        unsafe {
            self.adc_reg
                .adcctl0
                .set_bits(|w| w.adcenc().set_bit().adcsc().set_bit());
        }
    }

    /// Whether the ADC is currently sampling or converting.
    pub fn adc_is_busy(&self) -> bool {
        self.adc_reg.adcctl1.read().adcbusy().bit_is_set()
    }

    /// Gets the latest ADC conversion result.
    pub fn adc_get_result(&self) -> u16 {
        self.adc_reg.adcmem0.read().bits()
    }

    /// Selects which pin to sample. Can only be modified when the ADC is not busy.
    pub fn adc_set_pin<PIN>(&mut self, _pin: &PIN)
    where
        PIN: Channel<Adc<ADC>, ID = u8>,
    {
        self.adc_reg
            .adcmctl0
            .modify(|_, w| w.adcinch().bits(PIN::channel()));
    }
}

impl<WORD, PIN> OneShot<Adc<ADC>, WORD, PIN> for Adc<ADC>
where
    WORD: From<u16>,
    PIN: Channel<Adc<ADC>, ID = u8>,
{
    type Error = Infallible; // Only returns WouldBlock

    /// Begins a single ADC conversion if one is not already underway.
    ///
    /// If the result is ready it is returned, otherwise returns `WouldBlock`
    fn read(&mut self, pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        if self.is_waiting {
            if self.adc_is_busy() {
                return Err(nb::Error::WouldBlock);
            } else {
                self.is_waiting = false;
                return Ok(self.adc_get_result().into());
            }
        }

        self.adc_disable();
        self.adc_set_pin(pin);
        self.adc_enable();

        self.adc_start_conversion();
        self.is_waiting = true;
        Err(nb::Error::WouldBlock)
    }
}
