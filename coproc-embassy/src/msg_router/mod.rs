use crate::cobs_buffer::CobsBuffer;
use crate::usb::{Disconnected, UsbResponder};
use embassy_usb::class::cdc_acm::Receiver as UsbReceiver;

pub mod cmds;
use cmds::*;
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
                    self.handle_decoded(&incoming_buf[..decoded_bytes], &mut unencoded_buf)
                        .await
                } else {
                    None
                };

            if let Some(unencoded_bytes) = unencoded_bytes {
                self.responder
                    .send(&unencoded_buf[..(unencoded_bytes as usize)])
                    .await?;
            }
        }
    }

    async fn handle_decoded(
        &mut self,
        incoming_buf: &[u8],
        unencoded_buf: &mut [u8],
    ) -> Option<u8> {
        match (incoming_buf[0], incoming_buf[1], incoming_buf.len()) {
            (ping::MAJOR, ping::MINOR, _) => {
                unencoded_buf[0] = ping_response::MAJOR;
                unencoded_buf[1] = ping_response::MINOR;
                Some(2)
            }
            (update_row::MAJOR, update_row::MINOR, 4..) => {
                self.display_router
                    .handle_row_update(&incoming_buf[2..])
                    .await;
                None
            }
            (update_row_rgb::MAJOR, update_row_rgb::MINOR, _) => {
                self.display_router
                    .handle_row_update_rgb(&incoming_buf[2..])
                    .await;
                None
            }
            (get_display_info::MAJOR, get_display_info::MINOR, _) => {
                self.display_router.handle_get_display_info().await;
                None
            }
            (request_commit_render::MAJOR, request_commit_render::MINOR, _) => {
                unencoded_buf[0] = commit_render_response::MAJOR;
                unencoded_buf[1] = commit_render_response::MINOR;
                unencoded_buf[2] = 0x00;
                Some(3)
            }
            (set_single_cell::MAJOR, set_single_cell::MINOR, 5..) => {
                self.display_router
                    .handle_update_single_cell(&incoming_buf[2..])
                    .await;
                None
            }
            _ => None,
        }
    }
}
