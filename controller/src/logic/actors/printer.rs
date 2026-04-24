use crate::{printer::PrinterExt, story::Story};
use escpos::{driver::SerialPortDriver, printer::Printer};
use kameo::{Actor, prelude::Message};
use log::warn;
use miette::IntoDiagnostic;

#[derive(Actor)]
pub(crate) struct PrinterActor {
    printer: Printer<SerialPortDriver>,
}

impl PrinterActor {
    pub(crate) fn new(port: &str, baud_rate: u32) -> miette::Result<Self> {
        let printer = crate::printer::init(port, baud_rate).into_diagnostic()?;
        Ok(Self { printer })
    }
}

impl Message<Story> for PrinterActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: Story,
        _: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let Err(e) = self.printer.print_story(&msg) {
            warn!("Failed to print story: {e}");
        }
    }
}
