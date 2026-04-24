#![cfg_attr(not(feature = "std"), no_std)]

use core::{array::TryFromSliceError, time::Duration};
use defmt::Format;
use hex::{FromHex, FromHexError};
use serde::{Deserialize, Serialize};

pub const USB_VENDOR_ID: u16 = 0xFACE;
pub const USB_PRODUCT_ID: u16 = 0xF00D;

pub const USB_STATE_ENDPOINT: u8 = 0x80 | 0x01;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Uptime(core::time::Duration),
    CardReaderInit(CardReader),
    CardReader(CardReaderState),
    Button(ButtonState),
}

pub const USB_COMMAND_ENDPOINT: u8 = 0x01;

#[derive(Debug, Format, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    SetButtonLed(ButtonLedIntensity),
    SetLights(LedData),
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardReader {
    One,
    Two,
    Three,
    Four,
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardId([u8; 4]);

impl TryFrom<&[u8]> for CardId {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let value: [u8; 4] = value.try_into()?;
        Ok(Self(value))
    }
}

impl FromHex for CardId {
    type Error = FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let mut id = CardId([0u8; 4]);
        hex::decode_to_slice(hex, &mut id.0)?;
        Ok(id)
    }
}

impl AsRef<[u8]> for CardId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for CardId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardReaderState {
    pub reader: CardReader,
    pub state: Option<CardId>,
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonState {
    Pressed,
    Released(Duration),
}

#[derive(Debug, Format, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ButtonLedIntensity(u8);

impl ButtonLedIntensity {
    pub const OFF: Self = Self(0);
    pub const MAX: Self = Self(100);

    pub fn new(v: u8) -> Self {
        Self(v)
    }
}

impl core::ops::Deref for ButtonLedIntensity {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub const LED_COUNT: usize = 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedData {
    pub data: heapless::Vec<smart_leds::RGB8, LED_COUNT>,
}

impl Format for LedData {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "LedData(len = {})", self.data.len())
    }
}
