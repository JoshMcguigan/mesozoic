pub type DisplayColor = embedded_graphics::pixelcolor::Rgb565;

pub const LCD_W: u16 = 240;
pub const LCD_H: u16 = 240;

pub enum AppInput {
    AppleMedia(AppleMediaServiceData),
    Battery(BatteryData),
    Time(TimeOfDay),
    Touch(Touch),
    /// The platform should provide this input at the rate requested by the app.
    ///
    /// TODO see future AppOutput::TickRate
    /// app will request higher tick rate when the display/backlight is on, for example
    Tick,
}

pub struct AppleMediaServiceData {
    pub artist: AppleMediaServiceString,
    pub album: AppleMediaServiceString,
    pub title: AppleMediaServiceString,
}
const ATT_PAYLOAD_MAX_LEN: usize = 512;
pub type AppleMediaServiceString = arrayvec::ArrayString<ATT_PAYLOAD_MAX_LEN>;

pub struct BatteryData {
    pub charging: bool,
}

#[derive(Default, Clone)]
pub struct TimeOfDay {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

pub struct Touch {
    pub gesture: Gesture,
    pub event_type: EventType,
    pub x: u8,
    pub y: u8,
}

pub enum Gesture {
    None,
    SlideDown,
    SlideUp,
    SlideLeft,
    SlideRight,
    SingleClick,
    DoubleClick,
    LongPress,
}

pub enum EventType {
    Down,
    Up,
    Contact,
}

impl TryFrom<u8> for Gesture {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Gesture::None),
            0x01 => Ok(Gesture::SlideDown),
            0x02 => Ok(Gesture::SlideUp),
            0x03 => Ok(Gesture::SlideLeft),
            0x04 => Ok(Gesture::SlideRight),
            0x05 => Ok(Gesture::SingleClick),
            0x0B => Ok(Gesture::DoubleClick),
            0x0C => Ok(Gesture::LongPress),
            other => Err(other),
        }
    }
}

impl TryFrom<u8> for EventType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EventType::Down),
            1 => Ok(EventType::Up),
            2 => Ok(EventType::Contact),
            other => Err(other),
        }
    }
}
