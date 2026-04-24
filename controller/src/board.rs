use log::{debug, info, warn};
use miette::IntoDiagnostic;
use nusb::{
    io::{EndpointRead, EndpointWrite},
    transfer::{Bulk, In, Out},
};
use protocol::{
    Command, State, USB_COMMAND_ENDPOINT, USB_PRODUCT_ID, USB_STATE_ENDPOINT, USB_VENDOR_ID,
};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

#[derive(Default, Clone)]
pub(crate) struct Board {
    state_endpoint: Arc<Mutex<Option<EndpointRead<Bulk>>>>,
    command_endpoint: Arc<Mutex<Option<EndpointWrite<Bulk>>>>,
}

impl Board {
    pub(crate) async fn open(&self) {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            match open_device().await {
                Ok((command_endpoint, state_endpoint)) => {
                    {
                        let mut guard = self.command_endpoint.lock().await;
                        *guard = Some(command_endpoint);
                    }
                    {
                        let mut guard = self.state_endpoint.lock().await;
                        *guard = Some(state_endpoint);
                    }
                    return;
                }
                Err(e) => {
                    warn!("Failed to open device: {e}");
                    continue;
                }
            }
        }
    }

    pub(crate) async fn send_command(&self, cmd: Command) -> miette::Result<()> {
        let data = postcard::to_stdvec_cobs(&cmd).into_diagnostic()?;
        debug!("Sending command: {:?} ({} bytes)", cmd, data.len());

        let mut guard = self.command_endpoint.lock().await;

        let mut endpoint = guard
            .take()
            .ok_or(miette::miette!("Device not connected"))?;

        let result = async {
            endpoint.write_all(&data).await.into_diagnostic()?;
            endpoint.flush().await.into_diagnostic()?;
            Ok(())
        }
        .await;

        *guard = Some(endpoint);

        result
    }

    pub(crate) async fn receive_state(&self) -> miette::Result<State> {
        let mut guard = self.state_endpoint.lock().await;

        let mut endpoint = guard
            .take()
            .ok_or(miette::miette!("Device not connected"))?;

        let result = endpoint.receive_state().await;

        *guard = Some(endpoint);

        result
    }
}

async fn open_device() -> miette::Result<(EndpointWrite<Bulk>, EndpointRead<Bulk>)> {
    let di = nusb::list_devices()
        .await
        .into_diagnostic()?
        .find(|d| d.vendor_id() == USB_VENDOR_ID && d.product_id() == USB_PRODUCT_ID)
        .ok_or(miette::miette!("Device not found"))?;

    let device = di.open().await.into_diagnostic()?;

    let interface = device.claim_interface(0).await.into_diagnostic()?;

    let command_endpoint = interface
        .endpoint::<Bulk, Out>(USB_COMMAND_ENDPOINT)
        .into_diagnostic()?
        .writer(64);

    let state_endpoint = interface
        .endpoint::<Bulk, In>(USB_STATE_ENDPOINT)
        .into_diagnostic()?
        .reader(64);

    info!("Opened USB device");
    Ok((command_endpoint, state_endpoint))
}

trait StateEndpointExt {
    async fn receive_state(&mut self) -> miette::Result<State>;
}

impl StateEndpointExt for EndpointRead<Bulk> {
    async fn receive_state(&mut self) -> miette::Result<State> {
        let rx_buffer = &mut [0u8; 64];
        let payload_len = self.read(rx_buffer).await.into_diagnostic()?;

        let mut payload = rx_buffer[..payload_len].to_owned();
        debug!("Received bytes: {payload:?}");

        let state = postcard::from_bytes_cobs(&mut payload).into_diagnostic()?;
        debug!("Received: {state:?}");

        Ok(state)
    }
}
