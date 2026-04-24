use crate::{ButtonResources, usb_comm::STATE_CHANGE};
use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Input, Pull},
    pwm::{Pwm, SetDutyCycle},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Instant, Timer};
use protocol::{ButtonLedIntensity, ButtonState, State};

pub(crate) static BUTTON_LIGHT: Watch<CriticalSectionRawMutex, ButtonLedIntensity, 1> =
    Watch::new();

pub(super) fn init(r: ButtonResources, spawner: Spawner) {
    let button = Input::new(r.switch, Pull::Up);
    let led = Pwm::new_output_a(r.led_pwm, r.led, Default::default());

    spawner.must_spawn(input_task(button));
    spawner.must_spawn(led_task(led));
}

#[embassy_executor::task]
async fn input_task(mut button: Input<'static>) -> ! {
    let tx = STATE_CHANGE.sender();

    button.wait_for_high().await;
    tx.send(State::Button(ButtonState::Released(
        core::time::Duration::ZERO,
    )))
    .await;

    loop {
        button.wait_for_low().await;
        let time_pressed = Instant::now();
        info!("Button pressed");
        Timer::after_millis(50).await;
        if button.is_high() {
            warn!("Short press, ignoring");
            continue;
        }
        tx.send(State::Button(ButtonState::Pressed)).await;

        button.wait_for_high().await;
        let time_released = Instant::now();
        let duration: core::time::Duration = (time_released - time_pressed).into();
        info!("Button released after {}ms", duration.as_millis());
        tx.send(State::Button(ButtonState::Released(duration)))
            .await;
    }
}

#[embassy_executor::task]
async fn led_task(mut led: Pwm<'static>) -> ! {
    let mut rx = BUTTON_LIGHT.receiver().unwrap();

    loop {
        let intensity = *(rx.changed().await);
        info!("Setting button LED intensity to {}%", intensity);
        led.set_duty_cycle_percent(intensity).unwrap();
    }
}
