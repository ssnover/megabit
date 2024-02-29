use crate::cobs_buffer::CobsBuffer;
use crate::usb::{Disconnected, UsbResponder};
use embassy_usb::class::cdc_acm::Receiver as UsbReceiver;

pub mod display_cmd_router;
use display_cmd_router::DisplayCmdRouter;

pub struct MessageRouter<
    D: embassy_usb_driver::Driver<'static> + 'static,
    R: UsbResponder + 'static,
    const DECODE_BUFFER_SIZE: usize,
> {
    class: UsbReceiver<'static, D>,
    cobs_decoder: CobsBuffer<'static, DECODE_BUFFER_SIZE>,
    responder: &'static R,
    display_router: DisplayCmdRouter,
}

impl<
        D: embassy_usb_driver::Driver<'static> + 'static,
        R: UsbResponder + 'static,
        const DECODE_BUFFER_SIZE: usize,
    > MessageRouter<D, R, DECODE_BUFFER_SIZE>
{
    pub fn new(
        class: UsbReceiver<'static, D>,
        cobs_decoder: CobsBuffer<'static, DECODE_BUFFER_SIZE>,
        responder: &'static R,
        display_router: DisplayCmdRouter,
    ) -> Self {
        Self {
            class,
            cobs_decoder,
            responder,
            display_router,
        }
    }

    pub async fn run(mut self) {
        loop {
            self.wait_for_connection().await;
            let _ = self.handle_incoming().await;
        }
    }

    async fn wait_for_connection(&mut self) {
        self.class.wait_connection().await
    }

    async fn handle_incoming(&mut self) -> Result<(), Disconnected> {
        let mut incoming_buf = [0; 64];
        let mut unencoded_buf = [0; 64];
        loop {
            let bytes_read = self.class.read_packet(&mut incoming_buf).await?;
            self.cobs_decoder.write_bytes(&incoming_buf[..bytes_read]);
            let unencoded_bytes =
                if let Ok(decoded_bytes @ 2..) = self.cobs_decoder.read_packet(&mut incoming_buf) {
                    match (incoming_buf[0], incoming_buf[1], decoded_bytes) {
                        (0xde, 0x00, _) => {
                            unencoded_buf[0] = 0xde;
                            unencoded_buf[1] = 0x01;
                            Some(2)
                        }
                        (0xa0, 0x00, 4..) => {
                            self.display_router
                                .handle_row_update(&incoming_buf[2..])
                                .await;
                            None
                        }
                        (0xa0, 0x02, _) => {
                            unencoded_buf[0] = 0xa0;
                            unencoded_buf[1] = 0x03;
                            unencoded_buf[2] = 0x01;
                            Some(3)
                        }
                        (0xa0, 0x04, _) => {
                            unencoded_buf[0] = 0xa0;
                            unencoded_buf[1] = 0x05;
                            unencoded_buf[2..]
                                .iter_mut()
                                .zip(32u32.to_be_bytes().into_iter())
                                .for_each(|(dst, src)| *dst = src);
                            unencoded_buf[6..]
                                .iter_mut()
                                .zip(16u32.to_be_bytes().into_iter())
                                .for_each(|(dst, src)| *dst = src);
                            unencoded_buf[10] = 0x00;
                            Some(11)
                        }
                        (0xa0, 0x06, _) => {
                            unencoded_buf[0] = 0xa0;
                            unencoded_buf[1] = 0x07;
                            unencoded_buf[2] = 0x00;
                            Some(3)
                        }
                        (0xa0, 0x50, 5..) => {
                            self.display_router
                                .handle_update_single_cell(&incoming_buf[2..])
                                .await;
                            None
                        }
                        _ => None,
                    }
                } else {
                    None
                };

            if let Some(unencoded_bytes) = unencoded_bytes {
                self.responder
                    .send(&unencoded_buf[..unencoded_bytes])
                    .await?;
            }
        }
    }
}
