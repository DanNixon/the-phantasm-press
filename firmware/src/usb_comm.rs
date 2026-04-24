use crate::{UsbResources, button::BUTTON_LIGHT, leds::LED_DATA};
use defmt::{debug, info, warn};
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    peripherals::USB,
    usb::{Driver, InterruptHandler},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_usb::{Builder, Config, UsbDevice};
use embassy_usb_driver::{Endpoint, EndpointAddress, EndpointIn, EndpointOut};
use heapless::Vec;
use protocol::{
    Command, State, USB_COMMAND_ENDPOINT, USB_PRODUCT_ID, USB_STATE_ENDPOINT, USB_VENDOR_ID,
};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

pub(super) fn init(r: UsbResources, spawner: Spawner) {
    let driver = Driver::new(r.usb, Irqs);

    let mut config = Config::new(USB_VENDOR_ID, USB_PRODUCT_ID);
    config.manufacturer = Some("Dan Nixon");
    config.product = Some("The Phantasm Press Board");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

    let mut builder = Builder::new(
        driver,
        config,
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64]),
    );

    let mut function = builder.function(0xFF, 0, 0);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0xFF, 0, 0, None);

    let state_endpoint = alt.endpoint_bulk_in(Some(EndpointAddress::from(USB_STATE_ENDPOINT)), 64);
    let command_endpoint =
        alt.endpoint_bulk_out(Some(EndpointAddress::from(USB_COMMAND_ENDPOINT)), 64);

    drop(function);

    let usb = builder.build();

    spawner.must_spawn(usb_task(usb));

    spawner.must_spawn(state_task(state_endpoint));
    spawner.must_spawn(command_task(command_endpoint));
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    usb.run().await
}

pub(crate) static STATE_CHANGE: Channel<CriticalSectionRawMutex, State, 8> = Channel::new();

#[embassy_executor::task]
async fn state_task(
    mut endpoint: <embassy_rp::usb::Driver<'static, USB> as embassy_usb_driver::Driver<'static>>::EndpointIn,
) -> ! {
    let state_rx = STATE_CHANGE.receiver();

    loop {
        endpoint.wait_enabled().await;
        info!("Connected");

        'tx: loop {
            let state = state_rx.receive().await;
            debug!("Sending state: {:?}", state);

            match postcard::to_vec_cobs::<_, 64>(&state) {
                Ok(data) => {
                    if let Err(e) = endpoint.write(&data).await {
                        warn!("Failed to send state: {}", e);
                        break 'tx;
                    }
                }
                Err(_) => {
                    warn!("Failed to serialize state");
                }
            }
        }

        warn!("Disconnected");
    }
}

#[embassy_executor::task]
async fn command_task(
    mut endpoint: <embassy_rp::usb::Driver<'static, USB> as embassy_usb_driver::Driver<'static>>::EndpointOut,
) -> ! {
    let button_led_tx = BUTTON_LIGHT.sender();
    let led_data_tx = LED_DATA.sender();

    loop {
        endpoint.wait_enabled().await;
        info!("Connected");

        let mut command_buffer: Vec<u8, 256> = Vec::new();
        'rx: loop {
            let mut buffer = [0; 64];
            match endpoint.read(&mut buffer).await {
                Ok(n) => {
                    let data = &buffer[..n];
                    debug!("Received data (len {}): {}", data.len(), data);

                    command_buffer.extend_from_slice(data).unwrap();

                    if command_buffer.last() == Some(&0u8) {
                        debug!("Got full command buffer of length {}", command_buffer.len());

                        // Deserialize the command
                        match postcard::from_bytes_cobs::<Command>(&mut command_buffer) {
                            Ok(command) => {
                                info!("Received command: {:?}", command);

                                // Action the command
                                match command {
                                    Command::SetButtonLed(intensity) => {
                                        button_led_tx.send(intensity);
                                    }
                                    Command::SetLights(setting) => {
                                        if led_data_tx.try_send(setting).is_err() {
                                            warn!("Failed to send LED data on channel");
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                warn!("Failed to deserialize command");
                            }
                        }

                        command_buffer.clear();
                    }
                }
                Err(e) => {
                    warn!("Failed receive command data: {}", e);
                    break 'rx;
                }
            }
        }

        warn!("Disconnected");
    }
}
