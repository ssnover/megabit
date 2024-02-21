#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    peripherals, spim,
    usb::{self, vbus_detect::HardwareVbusDetect},
};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{Channel, Sender},
};
use embassy_usb::{
    class::cdc_acm::{self, CdcAcmClass},
    driver::EndpointError,
};
use megabit_coproc_embassy::{cobs_buffer::CobsBuffer, dot_matrix::DotMatrix};
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    POWER_CLOCK => usb::vbus_detect::InterruptHandler;
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    USBD => usb::InterruptHandler<peripherals::USBD>;
});

type UsbDriver = usb::Driver<'static, peripherals::USBD, HardwareVbusDetect>;

static SET_LED_CHANNEL: StaticCell<Channel<NoopRawMutex, (u8, u8, bool), 1>> = StaticCell::new();
static ROW_UPDATE_CHANNEL: StaticCell<Channel<NoopRawMutex, (u8, [u8; 4]), 4>> = StaticCell::new();

#[embassy_executor::task]
async fn usb_driver_task(mut device: embassy_usb::UsbDevice<'static, UsbDriver>) {
    device.run().await;
}

#[embassy_executor::task]
async fn ping_response_task(
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

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let nrf_peripherals = embassy_nrf::init(Default::default());
    let mut led = Output::new(nrf_peripherals.P0_06, Level::Low, OutputDrive::Standard);

    let usb_driver = usb::Driver::new(nrf_peripherals.USBD, Irqs, HardwareVbusDetect::new(Irqs));
    let mut config = embassy_usb::Config::new(0x16c0, 0x27de);
    config.manufacturer = Some("Snostorm Labs");
    config.product = Some("Megabit coproc");
    config.serial_number = Some("0123456789ABCDEF");
    config.max_power = 125;
    config.max_packet_size_0 = 64;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    static COBS_DECODE_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();
    static COBS_ENCODE_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
    let cobs_decoder = CobsBuffer::new(COBS_DECODE_BUFFER.init([0; 1024]));

    static STATE: StaticCell<cdc_acm::State> = StaticCell::new();
    let state = STATE.init(cdc_acm::State::new());

    static DEVICE_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESC: StaticCell<[u8; 128]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    let mut builder = embassy_usb::Builder::new(
        usb_driver,
        config,
        &mut DEVICE_DESC.init([0; 256])[..],
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut MSOS_DESC.init([0; 128])[..],
        &mut CONTROL_BUF.init([0; 128])[..],
    );
    let class = CdcAcmClass::new(&mut builder, state, 64);
    let usb = builder.build();

    let channel = SET_LED_CHANNEL.init(Channel::new());
    let row_update_channel = ROW_UPDATE_CHANNEL.init(Channel::new());

    spawner.spawn(usb_driver_task(usb)).unwrap();
    spawner
        .spawn(ping_response_task(
            class,
            cobs_decoder,
            COBS_ENCODE_BUFFER.init([0; 256]),
            channel.sender(),
            row_update_channel.sender(),
        ))
        .unwrap();

    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M4;
    config.mode = spim::MODE_0;

    let spim = spim::Spim::new_txonly(
        nrf_peripherals.SPI3,
        Irqs,
        nrf_peripherals.P0_13,
        nrf_peripherals.P1_01,
        config,
    );
    let ncs_0 = Output::new(nrf_peripherals.P0_27, Level::High, OutputDrive::Standard);
    let ncs_1 = Output::new(nrf_peripherals.P0_21, Level::High, OutputDrive::Standard);

    let mut dot_matrix = DotMatrix::new(spim, ncs_0, ncs_1).await.unwrap();

    let rx = row_update_channel.receiver();
    loop {
        let (row, row_data) = rx.receive().await;
        dot_matrix.update_row(row as usize, row_data).await.unwrap();
        if led.is_set_high() {
            led.set_low();
        } else {
            led.set_high();
        }
    }
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
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
