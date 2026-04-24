use crate::{CardReaderResources, usb_comm::STATE_CHANGE};
use defmt::{debug, info, warn};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Level, Output},
    peripherals::SPI0,
    spi::Spi,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Timer;
use esp_hal_mfrc522::consts::{PCDErrorCode, UidSize};
use protocol::{CardId, CardReader, CardReaderState, State};
use static_cell::StaticCell;

pub(super) fn init(r: CardReaderResources, spawner: Spawner) {
    let reader_spi = Spi::new(r.spi, r.clk, r.mosi, r.miso, r.tx_dma, r.rx_dma, {
        let mut c = embassy_rp::spi::Config::default();
        c.frequency = 1_000_000;
        c
    });
    static READER_SPI: StaticCell<SpiBus> = StaticCell::new();
    let reader_spi = READER_SPI.init(Mutex::new(reader_spi));

    spawner.must_spawn(task(
        CardReader::One,
        reader_spi,
        Output::new(r.cs_1, Level::High),
        Output::new(r.reset_1, Level::High),
    ));
    spawner.must_spawn(task(
        CardReader::Two,
        reader_spi,
        Output::new(r.cs_2, Level::High),
        Output::new(r.reset_2, Level::High),
    ));
    spawner.must_spawn(task(
        CardReader::Three,
        reader_spi,
        Output::new(r.cs_3, Level::High),
        Output::new(r.reset_3, Level::High),
    ));
    spawner.must_spawn(task(
        CardReader::Four,
        reader_spi,
        Output::new(r.cs_4, Level::High),
        Output::new(r.reset_4, Level::High),
    ));
}

type SpiBus = Mutex<CriticalSectionRawMutex, Spi<'static, SPI0, embassy_rp::spi::Async>>;

#[embassy_executor::task(pool_size = 4)]
async fn task(
    reader_id: CardReader,
    spi: &'static SpiBus,
    cs: Output<'static>,
    mut rst: Output<'static>,
) -> ! {
    let spi_device = SpiDevice::new(spi, cs);
    let driver = esp_hal_mfrc522::drivers::SpiDriver::new(spi_device);
    let mut reader = esp_hal_mfrc522::MFRC522::new(driver);

    let tx = STATE_CHANGE.sender();
    let mut current_card = None;

    'connection: loop {
        // Hard reset the reader.
        info!("Resetting reader {}", reader_id);
        rst.set_low();
        Timer::after_millis(100).await;
        rst.set_high();

        // Wait a bit before initialising the reader after reset.
        Timer::after_millis(500).await;

        // Initialise the reader and check it is working.
        info!("Initialising reader {}", reader_id);
        let _ = reader.pcd_init().await;
        let _ = reader.pcd_selftest().await;
        if !reader.pcd_is_init().await {
            warn!("Reader {} failed to initialise", reader_id);
            continue 'connection;
        }
        info!("Reader {} initialised", reader_id);
        tx.send(State::CardReaderInit(reader_id)).await;

        loop {
            match reader.picc_is_new_card_present().await {
                Ok(_) => {
                    let card = reader.get_card(UidSize::Four).await;
                    if let Ok(card) = card {
                        let uid: CardId = card.uid_bytes[..4].try_into().unwrap();
                        debug!("Reader {}: {}", reader_id, uid);

                        let last_card = current_card.replace(uid);
                        if last_card != current_card {
                            let state = CardReaderState {
                                reader: reader_id,
                                state: current_card,
                            };
                            info!("{}", state);
                            tx.send(State::CardReader(state)).await;
                        }
                    }

                    // Needed to continuously detect a card
                    let mut buff = [0; 18];
                    let mut byte_count = 18;
                    _ = reader.mifare_read(0, &mut buff, &mut byte_count).await;

                    _ = reader.picc_halta().await;
                }
                Err(PCDErrorCode::Timeout) => {
                    debug!("Reader {}: no card", reader_id);

                    if current_card.is_some() {
                        let state = CardReaderState {
                            reader: reader_id,
                            state: None,
                        };
                        info!("{}", state);
                        tx.send(State::CardReader(state)).await;
                    }

                    current_card = None;
                }
                Err(code) => {
                    let reason = match code {
                        PCDErrorCode::Collision => "Collision",
                        PCDErrorCode::InternalError => "Internal error",
                        PCDErrorCode::Invalid => "Invalid argument",
                        PCDErrorCode::NoRoom => "No room in buffer",
                        PCDErrorCode::Timeout => "Timeout",
                        PCDErrorCode::Error => "Error",
                        PCDErrorCode::CrcWrong => "CRC wrong",
                        PCDErrorCode::Unknown => "Unknown",
                        PCDErrorCode::MifareNack => "Mifare NACK",
                        PCDErrorCode::SpiError(_) => "SPI",
                        PCDErrorCode::I2cError(_) => "i2c",
                        PCDErrorCode::DriverError => "Driver",
                    };
                    warn!("Reader {} failed: {}", reader_id, reason);
                    continue 'connection;
                }
            };

            Timer::after_millis(500).await;
        }
    }
}
