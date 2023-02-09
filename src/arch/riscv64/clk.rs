use super::mmio::{read32, write32};
use super::sdelay;

pub const D1_CCU_BASE: usize = 0x02001000; //D1 CCU
const CCU_PLL_CPU_CTRL_REG: usize = 0x000;
const CCU_PLL_DDR_CTRL_REG: usize = 0x010;
const CCU_PLL_PERI0_CTRL_REG: usize = 0x020;
const CCU_PLL_PERI1_CTRL_REG: usize = 0x028; //NO USER
const CCU_PLL_GPU_CTRL_REG: usize = 0x030; //NO USER
const CCU_PLL_VIDEO0_CTRL_REG: usize = 0x040;
const CCU_PLL_VIDEO1_CTRL_REG: usize = 0x048;
const CCU_PLL_VE_CTRL: usize = 0x058;
const CCU_PLL_DE_CTRL: usize = 0x060;
const CCU_PLL_HSIC_CTRL: usize = 0x070;
const CCU_PLL_AUDIO0_CTRL_REG: usize = 0x078;
const CCU_PLL_AUDIO1_CTRL_REG: usize = 0x080;
const CCU_PLL_DDR_PAT0_CTRL_REG: usize = 0x110;
const CCU_PLL_DDR_PAT1_CTRL_REG: usize = 0x114;
const CCU_PLL_PERI0_PAT0_CTRL_REG: usize = 0x120;
const CCU_PLL_PERI0_PAT1_CTRL_REG: usize = 0x124;
const CCU_PLL_PERI1_PAT0_CTRL_REG: usize = 0x128;
const CCU_PLL_PERI1_PAT1_CTRL_REG: usize = 0x12c;
const CCU_PLL_GPU_PAT0_CTRL_REG: usize = 0x130;
const CCU_PLL_GPU_PAT1_CTRL_REG: usize = 0x134;
const CCU_PLL_VIDEO0_PAT0_CTRL_REG: usize = 0x140;
const CCU_PLL_VIDEO0_PAT1_CTRL_REG: usize = 0x144;
const CCU_PLL_VIDEO1_PAT0_CTRL_REG: usize = 0x148;
const CCU_PLL_VIDEO1_PAT1_CTRL_REG: usize = 0x14c;
const CCU_PLL_VE_PAT0_CTRL_REG: usize = 0x158;
const CCU_PLL_VE_PAT1_CTRL_REG: usize = 0x15c;
const CCU_PLL_DE_PAT0_CTRL_REG: usize = 0x160;
const CCU_PLL_DE_PAT1_CTRL_REG: usize = 0x164;
const CCU_PLL_HSIC_PAT0_CTRL_REG: usize = 0x170;
const CCU_PLL_HSIC_PAT1_CTRL_REG: usize = 0x174;
const CCU_PLL_AUDIO0_PAT0_CTRL_REG: usize = 0x178;
const CCU_PLL_AUDIO0_PAT1_CTRL_REG: usize = 0x17c;
const CCU_PLL_AUDIO1_PAT0_CTRL_REG: usize = 0x180;
const CCU_PLL_AUDIO1_PAT1_CTRL_REG: usize = 0x184;
const CCU_PLL_CPU_BIAS_REG: usize = 0x300;
const CCU_PLL_DDR_BIAS_REG: usize = 0x310;
const CCU_PLL_PERI0_BIAS_REG: usize = 0x320;
const CCU_PLL_PERI1_BIAS_REG: usize = 0x328;
const CCU_PLL_GPU_BIAS_REG: usize = 0x330;
const CCU_PLL_VIDEO0_BIAS_REG: usize = 0x340;
const CCU_PLL_VIDEO1_BIAS_REG: usize = 0x348;
const CCU_PLL_VE_BIAS_REG: usize = 0x358;
const CCU_PLL_DE_BIAS_REG: usize = 0x360;
const CCU_PLL_HSIC_BIAS_REG: usize = 0x370;
const CCU_PLL_AUDIO0_BIAS_REG: usize = 0x378;
const CCU_PLL_AUDIO1_BIAS_REG: usize = 0x380;
const CCU_PLL_CPU_TUN_REG: usize = 0x400;
const CCU_CPU_AXI_CFG_REG: usize = 0x500;
const CCU_CPU_GATING_REG: usize = 0x504;
const CCU_PSI_CLK_REG: usize = 0x510;
const CCU_AHB3_CLK_REG: usize = 0x51c;
const CCU_APB0_CLK_REG: usize = 0x520;
const CCU_APB1_CLK_REG: usize = 0x524;
const CCU_MBUS_CLK_REG: usize = 0x540;
const CCU_DMA_BGR_REG: usize = 0x70c;
const CCU_DRAM_CLK_REG: usize = 0x800;
const CCU_MBUS_MAT_CLK_GATING_REG: usize = 0x804;
const CCU_DRAM_BGR_REG: usize = 0x80c;
pub const CCU_UART_BGR_REG: usize = 0x90C;
const CCU_TWI_BGR_REG: usize = 0x91C;
const CCU_SPI0_BGR_REG: usize = 0x940;
const CCU_SPI1_BGR_REG: usize = 0x944;
const CCU_SPI_BGR_REG: usize = 0x96C;
const CCU_RISCV_CLK_REG: usize = 0xd00;
const CCU_RISCV_GATING_REG: usize = 0xd04;
const CCU_RISCV_CFG_BGR_REG: usize = 0xd0c;

fn set_pll_cpux_axi() {
    let mut val;

    /* Select cpux clock src to osc24m, axi divide ratio is 3, system apb clk ratio is 4 */
    write32(
        D1_CCU_BASE + CCU_RISCV_CLK_REG,
        (0 << 24) | (3 << 8) | (1 << 0),
    );
    sdelay(1);

    /* Disable pll gating */
    val = read32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG);
    val &= !(1 << 27);
    write32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG, val);

    /* Enable pll ldo */
    val = read32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG);
    val |= 1 << 30;
    write32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG, val);
    sdelay(5);

    /* Set default clk to 1008mhz */
    val = read32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG);
    val &= !((0x3 << 16) | (0xff << 8) | (0x3 << 0));
    val |= 41 << 8;
    write32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG, val);

    /* Lock enable */
    val = read32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG);
    val |= 1 << 29;
    write32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG, val);

    /* Enable pll */
    val = read32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG);
    val |= 1 << 31;
    write32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG, val);

    /* Wait pll stable */
    while (read32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG) & (0x1 << 28)) == 0 {}
    sdelay(20);

    /* Enable pll gating */
    val = read32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG);
    val |= 1 << 27;
    write32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG, val);

    /* Lock disable */
    val = read32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG);
    val &= !(1 << 29);
    write32(D1_CCU_BASE + CCU_PLL_CPU_CTRL_REG, val);
    sdelay(1);

    /* Set and change cpu clk src */
    val = read32(D1_CCU_BASE + CCU_RISCV_CLK_REG);
    val &= !(0x07 << 24 | 0x3 << 8 | 0xf << 0);
    val |= 0x05 << 24 | 0x1 << 8;
    write32(D1_CCU_BASE + CCU_RISCV_CLK_REG, val);
    sdelay(1);
}

fn set_pll_periph0() {
    let mut val;

    /* Periph0 has been enabled */
    if read32(D1_CCU_BASE + CCU_PLL_PERI0_CTRL_REG) & (1 << 31) != 0 {
        return;
    }

    /* Change psi src to osc24m */
    val = read32(D1_CCU_BASE + CCU_PSI_CLK_REG);
    val &= !(0x3 << 24);
    write32(D1_CCU_BASE + CCU_PSI_CLK_REG, val);

    /* Set default val */
    write32(D1_CCU_BASE + CCU_PLL_PERI0_CTRL_REG, 0x63 << 8);

    /* Lock enable */
    val = read32(D1_CCU_BASE + CCU_PLL_PERI0_CTRL_REG);
    val |= 1 << 29;
    write32(D1_CCU_BASE + CCU_PLL_PERI0_CTRL_REG, val);

    /* Enabe pll 600m(1x) 1200m(2x) */
    val = read32(D1_CCU_BASE + CCU_PLL_PERI0_CTRL_REG);
    val |= 1 << 31;
    write32(D1_CCU_BASE + CCU_PLL_PERI0_CTRL_REG, val);

    /* Wait pll stable */
    while (read32(D1_CCU_BASE + CCU_PLL_PERI0_CTRL_REG) & (0x1 << 28)) == 0 {}
    sdelay(20);

    /* Lock disable */
    val = read32(D1_CCU_BASE + CCU_PLL_PERI0_CTRL_REG);
    val &= !(1 << 29);
    write32(D1_CCU_BASE + CCU_PLL_PERI0_CTRL_REG, val);
}

fn set_ahb() {
    write32(D1_CCU_BASE + CCU_PSI_CLK_REG, (2 << 0) | (0 << 8));
    write32(
        D1_CCU_BASE + CCU_PSI_CLK_REG,
        read32(D1_CCU_BASE + CCU_PSI_CLK_REG) | (0x03 << 24),
    );
    sdelay(1);
}

fn set_apb() {
    write32(D1_CCU_BASE + CCU_APB0_CLK_REG, (2 << 0) | (1 << 8));
    write32(
        D1_CCU_BASE + CCU_APB0_CLK_REG,
        (0x03 << 24) | read32(D1_CCU_BASE + CCU_APB0_CLK_REG),
    );
    sdelay(1);
}

fn set_dma() {
    /* Dma reset */
    write32(
        D1_CCU_BASE + CCU_DMA_BGR_REG,
        read32(D1_CCU_BASE + CCU_DMA_BGR_REG) | (1 << 16),
    );
    sdelay(20);
    /* Enable gating clock for dma */
    write32(
        D1_CCU_BASE + CCU_DMA_BGR_REG,
        read32(D1_CCU_BASE + CCU_DMA_BGR_REG) | (1 << 0),
    );
}

fn set_mbus() {
    let mut val;

    /* Reset mbus domain */
    val = read32(D1_CCU_BASE + CCU_MBUS_CLK_REG);
    val |= 0x1 << 30;
    write32(D1_CCU_BASE + CCU_MBUS_CLK_REG, val);
    sdelay(1);
}

fn set_module(addr: usize) {
    let mut val;

    if read32(addr) & (1 << 31) == 0 {
        val = read32(addr);
        write32(addr, val | (1 << 31) | (1 << 30));

        /* Lock enable */
        val = read32(addr);
        val |= 1 << 29;
        write32(addr, val);

        /* Wait pll stable */
        while (read32(addr) & (0x1 << 28)) == 0 {}
        sdelay(20);

        /* Lock disable */
        val = read32(addr);
        val &= !(1 << 29);
        write32(addr, val);
    }
}

pub fn clk_enable_module_uart(addr: usize, uart_num: u8) {
    let mut val;
    /* Open the clock gate for uart */
    val = read32(addr);
    val |= 1 << (0 + uart_num);
    write32(addr, val);

    /* Deassert uart reset */
    val = read32(addr);
    val |= 1 << (16 + uart_num);
    write32(addr, val);
}

pub fn init_clock() {
    set_pll_cpux_axi();
    set_pll_periph0();
    set_ahb();
    set_apb();
    set_dma();
    set_mbus();
    set_module(D1_CCU_BASE + CCU_PLL_PERI0_CTRL_REG);
    set_module(D1_CCU_BASE + CCU_PLL_VIDEO0_CTRL_REG);
    set_module(D1_CCU_BASE + CCU_PLL_VIDEO1_CTRL_REG);
    set_module(D1_CCU_BASE + CCU_PLL_VE_CTRL);
    set_module(D1_CCU_BASE + CCU_PLL_AUDIO0_CTRL_REG);
    set_module(D1_CCU_BASE + CCU_PLL_AUDIO1_CTRL_REG);
}
