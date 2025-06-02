//! DMA configuration for continuous Hub75 display refresh

use crate::config::*;
use crate::memory::DisplayMemory;
use embassy_rp::Peri;
use embassy_rp::pac::dma::regs::{ChTransCount, CtrlTrig};
use embassy_rp::pac::dma::vals::{DataSize, TreqSel};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3};

/// DMA channel assignment for Hub75 operation
///
/// Uses 4 DMA channels in a chained configuration for continuous operation:
/// - CH0: Transfer framebuffer data to data SM FIFO
/// - CH1: Reset CH0's read address for continuous loop
/// - CH2: Transfer BCM delay values to OE SM FIFO  
/// - CH3: Reset CH2's read address for continuous loop
pub struct Hub75DmaChannels<'d> {
    _fb_channel: Peri<'d, DMA_CH0>,
    _fb_loop_channel: Peri<'d, DMA_CH1>,
    _oe_channel: Peri<'d, DMA_CH2>,
    _oe_loop_channel: Peri<'d, DMA_CH3>,
}

impl<'d> Hub75DmaChannels<'d> {
    /// Initialize DMA channels for Hub75 operation
    ///
    /// Sets up a complex chained DMA configuration that:
    /// 1. Continuously feeds pixel data to the data state machine
    /// 2. Continuously feeds timing delays to the output enable state machine
    /// 3. Automatically reloads buffer pointers for seamless operation
    pub fn new(
        dma_channels: (
            Peri<'d, DMA_CH0>,
            Peri<'d, DMA_CH1>,
            Peri<'d, DMA_CH2>,
            Peri<'d, DMA_CH3>,
        ),
        memory: &DisplayMemory,
    ) -> Self {
        let (fb_channel, fb_loop_channel, oe_channel, oe_loop_channel) = dma_channels;

        Self::setup_framebuffer_dma(&fb_channel, &fb_loop_channel, memory);
        Self::setup_oe_dma(&oe_channel, &oe_loop_channel, memory);

        Self {
            _fb_channel: fb_channel,
            _fb_loop_channel: fb_loop_channel,
            _oe_channel: oe_channel,
            _oe_loop_channel: oe_loop_channel,
        }
    }

    /// Setup DMA channels for framebuffer transfer
    ///
    /// Channel 0: Transfers framebuffer data to PIO data SM
    /// Channel 1: Reloads channel 0's read address for continuous operation
    fn setup_framebuffer_dma(
        _fb_channel: &Peri<'d, DMA_CH0>,
        _fb_loop_channel: &Peri<'d, DMA_CH1>,
        memory: &DisplayMemory,
    ) {
        let dma = embassy_rp::pac::DMA;

        // Get PIO0 SM0 FIFO address
        let pio0 = embassy_rp::pac::PIO0;
        let data_fifo_addr = pio0.txf(0).as_ptr() as u32;

        // Channel 0: Transfer framebuffer to data SM
        let mut ch0_ctrl = CtrlTrig(0);
        ch0_ctrl.set_incr_read(true); // Increment read address
        ch0_ctrl.set_incr_write(false); // Don't increment write address (FIFO)
        ch0_ctrl.set_data_size(DataSize::SIZE_WORD); // 32-bit transfers
        ch0_ctrl.set_treq_sel(TreqSel::from_bits(dma_dreq::DATA_SM)); // PIO0_TX0
        ch0_ctrl.set_chain_to(1); // Chain to channel 1 when complete
        ch0_ctrl.set_irq_quiet(true); // Don't generate interrupts
        ch0_ctrl.set_en(true); // Enable channel

        dma.ch(0).al1_ctrl().write_value(ch0_ctrl.0);
        dma.ch(0)
            .read_addr()
            .write_value(memory.get_active_buffer_ptr() as u32);
        dma.ch(0).write_addr().write_value(data_fifo_addr);
        dma.ch(0)
            .trans_count()
            .write_value(ChTransCount((FRAME_SIZE / 4) as u32));

        // Channel 1: Reset channel 0's read address for continuous operation
        let mut ch1_ctrl = CtrlTrig(0);
        ch1_ctrl.set_incr_read(false); // Don't increment read address  
        ch1_ctrl.set_incr_write(false); // Don't increment write address
        ch1_ctrl.set_data_size(DataSize::SIZE_WORD); // 32-bit transfers
        ch1_ctrl.set_treq_sel(TreqSel::PERMANENT); // Transfer immediately
        ch1_ctrl.set_chain_to(0); // Chain back to channel 0
        ch1_ctrl.set_irq_quiet(true); // Don't generate interrupts
        ch1_ctrl.set_en(true); // Enable channel

        dma.ch(1).al1_ctrl().write_value(ch1_ctrl.0);
        dma.ch(1)
            .read_addr()
            .write_value(memory.get_fb_ptr_addr() as u32);
        dma.ch(1)
            .write_addr()
            .write_value(dma.ch(0).read_addr().as_ptr() as u32);
        dma.ch(1).trans_count().write_value(ChTransCount(1));
    }

    /// Setup DMA channels for output enable timing
    ///
    /// Channel 2: Transfers BCM delay values to PIO OE SM
    /// Channel 3: Reloads channel 2's read address for continuous operation
    fn setup_oe_dma(
        _oe_channel: &Peri<'d, DMA_CH2>,
        _oe_loop_channel: &Peri<'d, DMA_CH3>,
        memory: &DisplayMemory,
    ) {
        let dma = embassy_rp::pac::DMA;

        // Get PIO0 SM2 FIFO address
        let pio0 = embassy_rp::pac::PIO0;
        let oe_fifo_addr = pio0.txf(2).as_ptr() as u32;

        // Channel 2: Transfer delay values to OE SM
        let mut ch2_ctrl = CtrlTrig(0);
        ch2_ctrl.set_incr_read(true); // Increment read address
        ch2_ctrl.set_incr_write(false); // Don't increment write address (FIFO)
        ch2_ctrl.set_data_size(DataSize::SIZE_WORD); // 32-bit transfers
        ch2_ctrl.set_treq_sel(TreqSel::from_bits(dma_dreq::OE_SM)); // PIO0_TX2
        ch2_ctrl.set_chain_to(3); // Chain to channel 3 when complete
        ch2_ctrl.set_irq_quiet(true); // Don't generate interrupts
        ch2_ctrl.set_en(true); // Enable channel

        dma.ch(2).al1_ctrl().write_value(ch2_ctrl.0);
        dma.ch(2)
            .read_addr()
            .write_value(memory.get_delay_ptr() as u32);
        dma.ch(2).write_addr().write_value(oe_fifo_addr);
        dma.ch(2)
            .trans_count()
            .write_value(ChTransCount(COLOR_BITS as u32));

        // Channel 3: Reset channel 2's read address for continuous operation
        let mut ch3_ctrl = CtrlTrig(0);
        ch3_ctrl.set_incr_read(false); // Don't increment read address
        ch3_ctrl.set_incr_write(false); // Don't increment write address
        ch3_ctrl.set_data_size(DataSize::SIZE_WORD); // 32-bit transfers
        ch3_ctrl.set_treq_sel(TreqSel::PERMANENT); // Transfer immediately
        ch3_ctrl.set_chain_to(2); // Chain back to channel 2
        ch3_ctrl.set_irq_quiet(true); // Don't generate interrupts  
        ch3_ctrl.set_en(true); // Enable channel

        dma.ch(3).al1_ctrl().write_value(ch3_ctrl.0);
        dma.ch(3)
            .read_addr()
            .write_value(memory.get_delay_ptr_addr() as u32);
        dma.ch(3)
            .write_addr()
            .write_value(dma.ch(2).read_addr().as_ptr() as u32);
        dma.ch(3).trans_count().write_value(ChTransCount(1));
    }
}

/// DMA status information for debugging
#[derive(Debug, Clone, Copy)]
pub struct DmaStatus {
    pub ch0_busy: bool,
    pub ch1_busy: bool,
    pub ch2_busy: bool,
    pub ch3_busy: bool,
    pub ch0_trans_count: u32,
    pub ch2_trans_count: u32,
}

impl DmaStatus {
    /// Check if all DMA channels are operating correctly
    pub fn is_healthy(&self) -> bool {
        // At least one of the main channels should be busy
        (self.ch0_busy || self.ch2_busy) &&
            // Transfer counts should be reasonable
            self.ch0_trans_count < (FRAME_SIZE as u32) &&
            self.ch2_trans_count < (COLOR_BITS as u32)
    }
}
