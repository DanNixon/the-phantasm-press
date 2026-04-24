#![no_std]
#![no_main]

mod button;
mod card_readers;
mod leds;
mod usb_comm;

use assign_resources::assign_resources;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Executor;
use embassy_rp::{
    Peri,
    gpio::{Level, Output},
    multicore::{Stack, spawn_core1},
    peripherals,
    watchdog::Watchdog,
};
use embassy_time::{Duration, Instant, Timer};
use panic_probe as _;
use portable_atomic as _;
use static_cell::StaticCell;

use crate::usb_comm::STATE_CHANGE;

assign_resources! {
    system: SystemResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
    usb: UsbResources {
        usb: USB,
    },
    card_readers: CardReaderResources {
        miso: PIN_4,
        mosi: PIN_3,
        clk: PIN_2,
        spi: SPI0,
        tx_dma: DMA_CH0,
        rx_dma: DMA_CH1,

        cs_1: PIN_5,
        reset_1: PIN_6,

        cs_2: PIN_7,
        reset_2: PIN_8,

        cs_3: PIN_9,
        reset_3: PIN_10,

        cs_4: PIN_11,
        reset_4: PIN_12,
    },
    button: ButtonResources {
        switch: PIN_13,
        led: PIN_14,
        led_pwm: PWM_SLICE7,
    },
    leds: LedResources {
        data_pin: PIN_17,
        pio: PIO0,
        dma: DMA_CH2,
    },
}

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    info!("Hello, world!");
    info!("Git revision: {}", git_version::git_version!());

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                usb_comm::init(r.usb, spawner);
                button::init(r.button, spawner);
                spawner.must_spawn(leds::task(r.leds));
                spawner.must_spawn(system_task(r.system));
            });
        },
    );

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        card_readers::init(r.card_readers, spawner);
    });
}

#[embassy_executor::task]
async fn system_task(r: SystemResources) -> ! {
    let mut led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_secs(3));

    let state_tx = STATE_CHANGE.sender();

    loop {
        led.toggle();
        watchdog.feed();

        state_tx
            .send(protocol::State::Uptime(
                Instant::now().duration_since(Instant::MIN).into(),
            ))
            .await;

        Timer::after_millis(500).await;
    }
}
