use crate::LedResources;
use defmt::warn;
use embassy_rp::{
    bind_interrupts,
    peripherals::PIO0,
    pio::Pio,
    pio_programs::ws2812::{PioWs2812, PioWs2812Program},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Timer;
use protocol::{LED_COUNT, LedData};

pub(crate) static LED_DATA: Channel<CriticalSectionRawMutex, LedData, 8> = Channel::new();

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
});

#[embassy_executor::task]
pub(super) async fn task(r: LedResources) -> ! {
    let Pio {
        mut common, sm0, ..
    } = Pio::new(r.pio, Irqs);

    let mut led_data = [smart_leds::RGB8::default(); LED_COUNT];

    let program = PioWs2812Program::new(&mut common);
    let mut ws2812 =
        PioWs2812::<_, _, LED_COUNT, _>::new(&mut common, sm0, r.dma, r.data_pin, &program);

    let data_rx = LED_DATA.receiver();

    // Flash each pixel
    for i in 0..LED_COUNT {
        led_data.fill(smart_leds::RGB8::default());
        led_data[i] = smart_leds::RGB8::new(64, 0, 0);
        ws2812.write(&led_data).await;
        Timer::after_millis(5).await
    }

    // Set all pixels off after boot
    led_data.fill(smart_leds::RGB8::default());
    ws2812.write(&led_data).await;

    loop {
        let data = data_rx.receive().await;

        if data.data.len() > LED_COUNT {
            warn!(
                "Ignoring LED data message with {} pixels when number in driver is {}",
                data.data.len(),
                LED_COUNT
            );
            continue;
        }

        for (i, pixel) in data.data.iter().enumerate() {
            led_data[i] = *pixel;
        }

        ws2812.write(&led_data).await;
    }
}
