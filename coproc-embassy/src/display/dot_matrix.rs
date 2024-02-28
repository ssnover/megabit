use core::convert::Infallible;
use embedded_hal::{digital::OutputPin, spi::Error};
use embedded_hal_async::spi::SpiBus;

pub const COLUMNS: usize = 32;
pub const ROWS: usize = 16;

pub struct DotMatrix<SPIBUS: SpiBus, OUT1: OutputPin, OUT2: OutputPin> {
    spi_driver: SPIBUS,
    ncs_0: OUT1,
    ncs_1: OUT2,
    state_buffer: [[u8; 4]; 16],
}

impl<
        SPIERR: Error,
        SPIBUS: SpiBus<Error = SPIERR>,
        OUT1: OutputPin<Error = Infallible>,
        OUT2: OutputPin<Error = Infallible>,
    > DotMatrix<SPIBUS, OUT1, OUT2>
{
    pub async fn new(spi_driver: SPIBUS, ncs_0: OUT1, ncs_1: OUT2) -> Result<Self, SPIERR> {
        let mut matrix = DotMatrix {
            spi_driver,
            ncs_0,
            ncs_1,
            state_buffer: [[0u8; 4]; 16],
        };
        matrix.init().await?;
        matrix.clear().await?;

        Ok(matrix)
    }

    async fn init(&mut self) -> Result<(), SPIERR> {
        for ncs in [
            &mut self.ncs_0 as &mut dyn OutputPin<Error = Infallible>,
            &mut self.ncs_1,
        ] {
            Self::init_display(ncs, &mut self.spi_driver).await?;
        }

        Ok(())
    }

    pub async fn clear(&mut self) -> Result<(), SPIERR> {
        for ncs in [
            &mut self.ncs_0 as &mut dyn OutputPin<Error = Infallible>,
            &mut self.ncs_1,
        ] {
            Self::clear_display(ncs, &mut self.spi_driver).await?;
        }

        Ok(())
    }

    pub async fn set_pixel(&mut self, row: usize, col: usize, state: bool) -> Result<(), SPIERR> {
        if state {
            self.state_buffer[row][col / 8] |= 1 << (col % 8);
        } else {
            self.state_buffer[row][col / 8] &= !(1 << (col % 8));
        }
        let (ncs, subrow) = if (0..8).contains(&row) {
            (
                &mut self.ncs_0 as &mut dyn OutputPin<Error = Infallible>,
                row,
            )
        } else {
            (
                &mut self.ncs_1 as &mut dyn OutputPin<Error = Infallible>,
                row - 8,
            )
        };
        Self::update_display_row(ncs, &mut self.spi_driver, subrow, &self.state_buffer[row]).await
    }

    pub async fn update_row(&mut self, row: usize, row_data: [u8; 4]) -> Result<(), SPIERR> {
        self.state_buffer[row] = row_data;
        let (ncs, subrow) = if (0..8).contains(&row) {
            (
                &mut self.ncs_0 as &mut dyn OutputPin<Error = Infallible>,
                row,
            )
        } else {
            (
                &mut self.ncs_1 as &mut dyn OutputPin<Error = Infallible>,
                row - 8,
            )
        };
        Self::update_display_row(ncs, &mut self.spi_driver, subrow, &self.state_buffer[row]).await
    }

    async fn init_display(
        ncs: &mut dyn OutputPin<Error = Infallible>,
        spi: &mut SPIBUS,
    ) -> Result<(), SPIERR> {
        // Disable display test
        ncs.set_low().unwrap();
        spi.transfer(&mut [], &[0x0f, 0x00, 0x0f, 0x00, 0x0f, 0x00, 0x0f, 0x00])
            .await?;
        ncs.set_high().unwrap();

        // Set scan limit to max (7)
        ncs.set_low().unwrap();
        spi.transfer(&mut [], &[0x0b, 0x07, 0x0b, 0x07, 0x0b, 0x07, 0x0b, 0x07])
            .await?;
        ncs.set_high().unwrap();

        // Disable decode mode
        ncs.set_low().unwrap();
        spi.transfer(&mut [], &[0x09, 0x00, 0x09, 0x00, 0x09, 0x00, 0x09, 0x00])
            .await?;
        ncs.set_high().unwrap();

        // Set the brightness to low
        ncs.set_low().unwrap();
        spi.transfer(&mut [], &[0x0a, 0x03, 0x0a, 0x03, 0x0a, 0x03, 0x0a, 0x03])
            .await?;
        ncs.set_high().unwrap();

        // Disable shutdown mode
        ncs.set_low().unwrap();
        spi.transfer(&mut [], &[0x0c, 0x01, 0x0c, 0x01, 0x0c, 0x01, 0x0c, 0x01])
            .await?;
        ncs.set_high().unwrap();

        Ok(())
    }

    async fn clear_display(
        ncs: &mut dyn OutputPin<Error = Infallible>,
        spi: &mut SPIBUS,
    ) -> Result<(), SPIERR> {
        for row in 0..8 {
            ncs.set_low().unwrap();
            spi.transfer(
                &mut [],
                &[row + 1, 0x00, row + 1, 0x00, row + 1, 0x00, row + 1, 0x00],
            )
            .await?;
            ncs.set_high().unwrap();
        }

        Ok(())
    }

    async fn update_display_row(
        ncs: &mut dyn OutputPin<Error = Infallible>,
        spi: &mut SPIBUS,
        row: usize,
        row_data: &[u8; 4],
    ) -> Result<(), SPIERR> {
        let opcode = (7 - row as u8) + 1;
        let cmd_data = [
            opcode,
            row_data[3],
            opcode,
            row_data[2],
            opcode,
            row_data[1],
            opcode,
            row_data[0],
        ];

        ncs.set_low().unwrap();
        spi.transfer(&mut [], &cmd_data).await?;
        ncs.set_high().unwrap();

        Ok(())
    }
}
