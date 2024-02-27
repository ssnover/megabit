use crate::cobs_buffer::CobsBuffer;
use crate::usb::{Disconnected, UsbDriver};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Sender};
use embassy_usb::class::cdc_acm::CdcAcmClass;

#[embassy_executor::task]
pub async fn msg_handler_task(
    mut class: CdcAcmClass<'static, UsbDriver>,
    mut cobs_decoder: CobsBuffer<'static, 1024>,
    encode_buffer: &'static mut [u8; 256],
    mut led_sender: Sender<'static, NoopRawMutex, (u8, u8, bool), 1>,
    mut row_update_sender: Sender<'static, NoopRawMutex, (u8, [u8; 4]), 4>,
) {
    loop {
        class.wait_connection().await;
        let _ = handle_msg(
            &mut class,
            &mut cobs_decoder,
            encode_buffer,
            &mut led_sender,
            &mut row_update_sender,
        )
        .await;
    }
}

async fn handle_msg(
    class: &mut CdcAcmClass<'static, UsbDriver>,
    decode_buffer: &mut CobsBuffer<'static, 1024>,
    encode_buffer: &mut [u8; 256],
    led_sender: &mut Sender<'static, NoopRawMutex, (u8, u8, bool), 1>,
    row_update_sender: &mut Sender<'static, NoopRawMutex, (u8, [u8; 4]), 4>,
) -> Result<(), Disconnected> {
    let mut incoming_buf = [0; 64];
    let mut encoded_buf = [0; 64];
    loop {
        let bytes_read = class.read_packet(&mut incoming_buf).await?;
        decode_buffer.write_bytes(&incoming_buf[..bytes_read]);
        let unencoded_bytes =
            if let Ok(decoded_bytes @ 2..) = decode_buffer.read_packet(&mut incoming_buf) {
                match (incoming_buf[0], incoming_buf[1], decoded_bytes) {
                    (0xde, 0x00, _) => {
                        encode_buffer[0] = 0xde;
                        encode_buffer[1] = 0x01;
                        Some(2)
                    }
                    (0xa0, 0x00, 4..) => {
                        let row_number = incoming_buf[2];
                        let _row_data_len = incoming_buf[3];
                        let mut row_data = [0u8; 4];
                        row_data.clone_from_slice(&incoming_buf[4..8]);

                        row_update_sender.send((row_number, row_data)).await;

                        encode_buffer[0] = 0xa0;
                        encode_buffer[1] = 0x01;
                        encode_buffer[2] = 0x00;
                        Some(3)
                    }
                    (0xa0, 0x02, _) => {
                        encode_buffer[0] = 0xa0;
                        encode_buffer[1] = 0x03;
                        encode_buffer[2] = 0x01;
                        Some(3)
                    }
                    (0xa0, 0x04, _) => {
                        encode_buffer[0] = 0xa0;
                        encode_buffer[1] = 0x05;
                        encode_buffer[2..]
                            .iter_mut()
                            .zip(32u32.to_be_bytes().into_iter())
                            .for_each(|(dst, src)| *dst = src);
                        encode_buffer[6..]
                            .iter_mut()
                            .zip(16u32.to_be_bytes().into_iter())
                            .for_each(|(dst, src)| *dst = src);
                        encode_buffer[10] = 0x00;
                        Some(11)
                    }
                    (0xa0, 0x06, _) => {
                        encode_buffer[0] = 0xa0;
                        encode_buffer[1] = 0x07;
                        encode_buffer[2] = 0x00;
                        Some(3)
                    }
                    (0xa0, 0x50, 5..) => {
                        led_sender
                            .send((incoming_buf[2], incoming_buf[3], incoming_buf[4] != 0x00))
                            .await;
                        encode_buffer[0] = 0xa0;
                        encode_buffer[1] = 0x51;
                        Some(2)
                    }
                    _ => None,
                }
            } else {
                None
            };

        if let Some(unencoded_bytes) = unencoded_bytes {
            let encoded_bytes = cobs::encode(&encode_buffer[..unencoded_bytes], &mut encoded_buf);
            encoded_buf[encoded_bytes] = 0x00;
            class
                .write_packet(&encoded_buf[..encoded_bytes + 1])
                .await?;
        }
    }
}
