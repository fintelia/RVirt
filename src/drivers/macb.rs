
// References:
//
// https://github.com/qemu/qemu/blob/d522fba24478474911b0e6e488b6d1dcf1af54f8/hw/net/cadence_gem.c
// https://github.com/torvalds/linux/blob/master/drivers/net/ethernet/cadence/macb_main.c
// https://www.yumpu.com/en/document/view/31739994/gigabit-ethernet-mac-gem-technical-data-sheet-cadence-

use crate::memory_region::MemoryRegion;
use crate::pmap;
use super::*;

// Values taken for the QEMU source code.
#[allow(unused)]
mod constants {
    pub const GEM_NWCTRL: u64 = 0x00000000/4; /* Network Control reg */
    pub const GEM_NWCFG: u64 = 0x00000004/4; /* Network Config reg */
    pub const GEM_NWSTATUS: u64 = 0x00000008/4; /* Network Status reg */
    pub const GEM_USERIO: u64 = 0x0000000C/4; /* User IO reg */
    pub const GEM_DMACFG: u64 = 0x00000010/4; /* DMA Control reg */
    pub const GEM_TXSTATUS: u64 = 0x00000014/4; /* TX Status reg */
    pub const GEM_RXQBASE: u64 = 0x00000018/4; /* RX Q Base address reg */
    pub const GEM_TXQBASE: u64 = 0x0000001C/4; /* TX Q Base address reg */
    pub const GEM_RXSTATUS: u64 = 0x00000020/4; /* RX Status reg */
    pub const GEM_ISR: u64 = 0x00000024/4; /* Interrupt Status reg */
    pub const GEM_IER: u64 = 0x00000028/4; /* Interrupt Enable reg */
    pub const GEM_IDR: u64 = 0x0000002C/4; /* Interrupt Disable reg */
    pub const GEM_IMR: u64 = 0x00000030/4; /* Interrupt Mask reg */
    pub const GEM_PHYMNTNC: u64 = 0x00000034/4; /* Phy Maintenance reg */
    pub const GEM_RXPAUSE: u64 = 0x00000038/4; /* RX Pause Time reg */
    pub const GEM_TXPAUSE: u64 = 0x0000003C/4; /* TX Pause Time reg */
    pub const GEM_TXPARTIALSF: u64 = 0x00000040/4; /* TX Partial Store and Forward */
    pub const GEM_RXPARTIALSF: u64 = 0x00000044/4; /* RX Partial Store and Forward */
    pub const GEM_HASHLO: u64 = 0x00000080/4; /* Hash Low address reg */
    pub const GEM_HASHHI: u64 = 0x00000084/4; /* Hash High address reg */
    pub const GEM_SPADDR1LO: u64 = 0x00000088/4; /* Specific addr 1 low reg */
    pub const GEM_SPADDR1HI: u64 = 0x0000008C/4; /* Specific addr 1 high reg */
    pub const GEM_SPADDR2LO: u64 = 0x00000090/4; /* Specific addr 2 low reg */
    pub const GEM_SPADDR2HI: u64 = 0x00000094/4; /* Specific addr 2 high reg */
    pub const GEM_SPADDR3LO: u64 = 0x00000098/4; /* Specific addr 3 low reg */
    pub const GEM_SPADDR3HI: u64 = 0x0000009C/4; /* Specific addr 3 high reg */
    pub const GEM_SPADDR4LO: u64 = 0x000000A0/4; /* Specific addr 4 low reg */
    pub const GEM_SPADDR4HI: u64 = 0x000000A4/4; /* Specific addr 4 high reg */
    pub const GEM_TIDMATCH1: u64 = 0x000000A8/4; /* Type ID1 Match reg */
    pub const GEM_TIDMATCH2: u64 = 0x000000AC/4; /* Type ID2 Match reg */
    pub const GEM_TIDMATCH3: u64 = 0x000000B0/4; /* Type ID3 Match reg */
    pub const GEM_TIDMATCH4: u64 = 0x000000B4/4; /* Type ID4 Match reg */
    pub const GEM_WOLAN: u64 = 0x000000B8/4; /* Wake on LAN reg */
    pub const GEM_IPGSTRETCH: u64 = 0x000000BC/4; /* IPG Stretch reg */
    pub const GEM_SVLAN: u64 = 0x000000C0/4; /* Stacked VLAN reg */
    pub const GEM_MODID: u64 = 0x000000FC/4; /* Module ID reg */
    pub const GEM_OCTTXLO: u64 = 0x00000100/4; /* Octects transmitted Low reg */
    pub const GEM_OCTTXHI: u64 = 0x00000104/4; /* Octects transmitted High reg */
    pub const GEM_TXCNT: u64 = 0x00000108/4; /* Error-free Frames transmitted */
    pub const GEM_TXBCNT: u64 = 0x0000010C/4; /* Error-free Broadcast Frames */
    pub const GEM_TXMCNT: u64 = 0x00000110/4; /* Error-free Multicast Frame */
    pub const GEM_TXPAUSECNT: u64 = 0x00000114/4; /* Pause Frames Transmitted */
    pub const GEM_TX64CNT: u64 = 0x00000118/4; /* Error-free 64 TX */
    pub const GEM_TX65CNT: u64 = 0x0000011C/4; /* Error-free 65-127 TX */
    pub const GEM_TX128CNT: u64 = 0x00000120/4; /* Error-free 128-255 TX */
    pub const GEM_TX256CNT: u64 = 0x00000124/4; /* Error-free 256-511 */
    pub const GEM_TX512CNT: u64 = 0x00000128/4; /* Error-free 512-1023 TX */
    pub const GEM_TX1024CNT: u64 = 0x0000012C/4; /* Error-free 1024-1518 TX */
    pub const GEM_TX1519CNT: u64 = 0x00000130/4; /* Error-free larger than 1519 TX */
    pub const GEM_TXURUNCNT: u64 = 0x00000134/4; /* TX under run error counter */
    pub const GEM_SINGLECOLLCNT: u64 = 0x00000138/4; /* Single Collision Frames */
    pub const GEM_MULTCOLLCNT: u64 = 0x0000013C/4; /* Multiple Collision Frames */
    pub const GEM_EXCESSCOLLCNT: u64 = 0x00000140/4; /* Excessive Collision Frames */
    pub const GEM_LATECOLLCNT: u64 = 0x00000144/4; /* Late Collision Frames */
    pub const GEM_DEFERTXCNT: u64 = 0x00000148/4; /* Deferred Transmission Frames */
    pub const GEM_CSENSECNT: u64 = 0x0000014C/4; /* Carrier Sense Error Counter */
    pub const GEM_OCTRXLO: u64 = 0x00000150/4; /* Octects Received register Low */
    pub const GEM_OCTRXHI: u64 = 0x00000154/4; /* Octects Received register High */
    pub const GEM_RXCNT: u64 = 0x00000158/4; /* Error-free Frames Received */
    pub const GEM_RXBROADCNT: u64 = 0x0000015C/4; /* Error-free Broadcast Frames RX */
    pub const GEM_RXMULTICNT: u64 = 0x00000160/4; /* Error-free Multicast Frames RX */
    pub const GEM_RXPAUSECNT: u64 = 0x00000164/4; /* Pause Frames Received Counter */
    pub const GEM_RX64CNT: u64 = 0x00000168/4; /* Error-free 64 byte Frames RX */
    pub const GEM_RX65CNT: u64 = 0x0000016C/4; /* Error-free 65-127B Frames RX */
    pub const GEM_RX128CNT: u64 = 0x00000170/4; /* Error-free 128-255B Frames RX */
    pub const GEM_RX256CNT: u64 = 0x00000174/4; /* Error-free 256-512B Frames RX */
    pub const GEM_RX512CNT: u64 = 0x00000178/4; /* Error-free 512-1023B Frames RX */
    pub const GEM_RX1024CNT: u64 = 0x0000017C/4; /* Error-free 1024-1518B Frames RX */
    pub const GEM_RX1519CNT: u64 = 0x00000180/4; /* Error-free 1519-max Frames RX */
    pub const GEM_RXUNDERCNT: u64 = 0x00000184/4; /* Undersize Frames Received */
    pub const GEM_RXOVERCNT: u64 = 0x00000188/4; /* Oversize Frames Received */
    pub const GEM_RXJABCNT: u64 = 0x0000018C/4; /* Jabbers Received Counter */
    pub const GEM_RXFCSCNT: u64 = 0x00000190/4; /* Frame Check seq. Error Counter */
    pub const GEM_RXLENERRCNT: u64 = 0x00000194/4; /* Length Field Error Counter */
    pub const GEM_RXSYMERRCNT: u64 = 0x00000198/4; /* Symbol Error Counter */
    pub const GEM_RXALIGNERRCNT: u64 = 0x0000019C/4; /* Alignment Error Counter */
    pub const GEM_RXRSCERRCNT: u64 = 0x000001A0/4; /* Receive Resource Error Counter */
    pub const GEM_RXORUNCNT: u64 = 0x000001A4/4; /* Receive Overrun Counter */
    pub const GEM_RXIPCSERRCNT: u64 = 0x000001A8/4; /* IP header Checksum Error Counter */
    pub const GEM_RXTCPCCNT: u64 = 0x000001AC/4; /* TCP Checksum Error Counter */
    pub const GEM_RXUDPCCNT: u64 = 0x000001B0/4; /* UDP Checksum Error Counter */

    pub const GEM_1588S: u64 = 0x000001D0/4; /* 1588 Timer Seconds */
    pub const GEM_1588NS: u64 = 0x000001D4/4; /* 1588 Timer Nanoseconds */
    pub const GEM_1588ADJ: u64 = 0x000001D8/4; /* 1588 Timer Adjust */
    pub const GEM_1588INC: u64 = 0x000001DC/4; /* 1588 Timer Increment */
    pub const GEM_PTPETXS: u64 = 0x000001E0/4; /* PTP Event Frame Transmitted (s) */
    pub const GEM_PTPETXNS: u64 = 0x000001E4/4; /* PTP Event Frame Transmitted (ns) */
    pub const GEM_PTPERXS: u64 = 0x000001E8/4; /* PTP Event Frame Received (s) */
    pub const GEM_PTPERXNS: u64 = 0x000001EC/4; /* PTP Event Frame Received (ns) */
    pub const GEM_PTPPTXS: u64 = 0x000001E0/4; /* PTP Peer Frame Transmitted (s) */
    pub const GEM_PTPPTXNS: u64 = 0x000001E4/4; /* PTP Peer Frame Transmitted (ns) */
    pub const GEM_PTPPRXS: u64 = 0x000001E8/4; /* PTP Peer Frame Received (s) */
    pub const GEM_PTPPRXNS: u64 = 0x000001EC/4; /* PTP Peer Frame Received (ns) */

    /* Design Configuration Registers */
    pub const GEM_DESCONF: u64 = 0x00000280/4;
    pub const GEM_DESCONF2: u64 = 0x00000284/4;
    pub const GEM_DESCONF3: u64 = 0x00000288/4;
    pub const GEM_DESCONF4: u64 = 0x0000028C/4;
    pub const GEM_DESCONF5: u64 = 0x00000290/4;
    pub const GEM_DESCONF6: u64 = 0x00000294/4;
    pub const GEM_DESCONF6_64B_MASK: u64 = 1 << 23;
    pub const GEM_DESCONF7: u64 = 0x00000298/4;

    pub const GEM_INT_Q1_STATUS: u64 = 0x00000400 / 4;

    pub const GEM_TRANSMIT_Q1_PTR: u64 = 0x00000440 / 4;
    pub const GEM_TRANSMIT_Q7_PTR: u64 = GEM_TRANSMIT_Q1_PTR + 6;

    pub const GEM_RECEIVE_Q1_PTR: u64 = 0x00000480 / 4;
    pub const GEM_RECEIVE_Q7_PTR: u64 = GEM_RECEIVE_Q1_PTR + 6;

    pub const GEM_TBQPH: u64 = 0x000004C8 / 4;
    pub const GEM_RBQPH: u64 = 0x000004D4 / 4;

    pub const GEM_INT_Q1_ENABLE: u64 = 0x00000600 / 4;
    pub const GEM_INT_Q7_ENABLE: u64 = GEM_INT_Q1_ENABLE + 6;

    pub const GEM_INT_Q1_DISABLE: u64 = 0x00000620 / 4;
    pub const GEM_INT_Q7_DISABLE: u64 = GEM_INT_Q1_DISABLE + 6;

    pub const GEM_INT_Q1_MASK: u64 = 0x00000640 / 4;
    pub const GEM_INT_Q7_MASK: u64 = GEM_INT_Q1_MASK + 6;

    pub const GEM_SCREENING_TYPE1_REGISTER_0: u64 = 0x00000500 / 4;

    pub const GEM_ST1R_UDP_PORT_MATCH_ENABLE: u64 = 1 << 29;
    pub const GEM_ST1R_DSTC_ENABLE: u64 = 1 << 28;
    pub const GEM_ST1R_UDP_PORT_MATCH_SHIFT: u64 = 12;
    pub const GEM_ST1R_UDP_PORT_MATCH_WIDTH: u64 = 27 - GEM_ST1R_UDP_PORT_MATCH_SHIFT + 1;
    pub const GEM_ST1R_DSTC_MATCH_SHIFT: u64 = 4;
    pub const GEM_ST1R_DSTC_MATCH_WIDTH: u64 = 11 - GEM_ST1R_DSTC_MATCH_SHIFT + 1;
    pub const GEM_ST1R_QUEUE_SHIFT: u64 = 0;
    pub const GEM_ST1R_QUEUE_WIDTH: u64 = 3 - GEM_ST1R_QUEUE_SHIFT + 1;

    pub const GEM_SCREENING_TYPE2_REGISTER_0: u64 = 0x00000540 / 4;

    pub const GEM_ST2R_COMPARE_A_ENABLE: u64 = 1 << 18;
    pub const GEM_ST2R_COMPARE_A_SHIFT: u64 = 13;
    pub const GEM_ST2R_COMPARE_WIDTH: u64 = 17 - GEM_ST2R_COMPARE_A_SHIFT + 1;
    pub const GEM_ST2R_ETHERTYPE_ENABLE: u64 = 1 << 12;
    pub const GEM_ST2R_ETHERTYPE_INDEX_SHIFT: u64 = 9;
    pub const GEM_ST2R_ETHERTYPE_INDEX_WIDTH: u64 = 11 - GEM_ST2R_ETHERTYPE_INDEX_SHIFT + 1;
    pub const GEM_ST2R_QUEUE_SHIFT: u64 = 0;
    pub const GEM_ST2R_QUEUE_WIDTH: u64 = 3 - GEM_ST2R_QUEUE_SHIFT + 1;

    pub const GEM_SCREENING_TYPE2_ETHERTYPE_REG_0: u64 = 0x000006e0 / 4;
    pub const GEM_TYPE2_COMPARE_0_WORD_0: u64 = 0x00000700 / 4;

    pub const GEM_T2CW1_COMPARE_OFFSET_SHIFT: u64 = 7;
    pub const GEM_T2CW1_COMPARE_OFFSET_WIDTH: u64 = 8 - GEM_T2CW1_COMPARE_OFFSET_SHIFT + 1;
    pub const GEM_T2CW1_OFFSET_VALUE_SHIFT: u64 = 0;
    pub const GEM_T2CW1_OFFSET_VALUE_WIDTH: u64 = 6 - GEM_T2CW1_OFFSET_VALUE_SHIFT + 1;

    /*****************************************/
    pub const GEM_NWCTRL_TXSTART: u64 = 0x00000200; /* Transmit Enable */
    pub const GEM_NWCTRL_TXENA: u64 = 0x00000008; /* Transmit Enable */
    pub const GEM_NWCTRL_RXENA: u64 = 0x00000004; /* Receive Enable */
    pub const GEM_NWCTRL_LOCALLOOP: u64 = 0x00000002; /* Local Loopback */

    pub const GEM_NWCFG_STRIP_FCS: u64 = 0x00020000; /* Strip FCS field */
    pub const GEM_NWCFG_LERR_DISC: u64 = 0x00010000; /* Discard RX frames with len err */
    pub const GEM_NWCFG_BUFF_OFST_M: u64 = 0x0000C000; /* Receive buffer offset mask */
    pub const GEM_NWCFG_BUFF_OFST_S: u64 = 14;         /* Receive buffer offset shift */
    pub const GEM_NWCFG_UCAST_HASH: u64 = 0x00000080; /* accept unicast if hash match */
    pub const GEM_NWCFG_MCAST_HASH: u64 = 0x00000040; /* accept multicast if hash match */
    pub const GEM_NWCFG_BCAST_REJ: u64 = 0x00000020; /* Reject broadcast packets */
    pub const GEM_NWCFG_PROMISC: u64 = 0x00000010; /* Accept all packets */

    pub const GEM_DMACFG_ADDR_64B: u64 = 1 << 30;
    pub const GEM_DMACFG_TX_BD_EXT: u64 = 1 << 29;
    pub const GEM_DMACFG_RX_BD_EXT: u64 = 1 << 28;
    pub const GEM_DMACFG_RBUFSZ_M: u64 = 0x00FF0000; /* DMA RX Buffer Size mask */
    pub const GEM_DMACFG_RBUFSZ_S: u64 = 16;         /* DMA RX Buffer Size shift */
    pub const GEM_DMACFG_RBUFSZ_MUL: u64 = 64;         /* DMA RX Buffer Size multiplier */
    pub const GEM_DMACFG_TXCSUM_OFFL: u64 = 0x00000800; /* Transmit checksum offload */

    pub const GEM_TXSTATUS_TXCMPL: u64 = 0x00000020; /* Transmit Complete */
    pub const GEM_TXSTATUS_USED: u64 = 0x00000001; /* sw owned descriptor encountered */

    pub const GEM_RXSTATUS_FRMRCVD: u64 = 0x00000002; /* Frame received */
    pub const GEM_RXSTATUS_NOBUF: u64 = 0x00000001; /* Buffer unavailable */

    /* GEM_ISR GEM_IER GEM_IDR GEM_IMR */
    pub const GEM_INT_TXCMPL: u64 = 0x00000080; /* Transmit Complete */
    pub const GEM_INT_TXUSED: u64 = 0x00000008;
    pub const GEM_INT_RXUSED: u64 = 0x00000004;
    pub const GEM_INT_RXCMPL: u64 = 0x00000002;

    pub const GEM_PHYMNTNC_OP_R: u64 = 0x20000000; /* read operation */
    pub const GEM_PHYMNTNC_OP_W: u64 = 0x10000000; /* write operation */
    pub const GEM_PHYMNTNC_ADDR: u64 = 0x0F800000; /* Address bits */
    pub const GEM_PHYMNTNC_ADDR_SHFT: u64 = 23;
    pub const GEM_PHYMNTNC_REG: u64 = 0x007C0000; /* register bits */
    pub const GEM_PHYMNTNC_REG_SHIFT: u64 = 18;

    /* Marvell PHY definitions */
    pub const BOARD_PHY_ADDRESS: u64 = 23; /* PHY address we will emulate a device at */

    pub const PHY_REG_CONTROL: u64 = 0;
    pub const PHY_REG_STATUS: u64 = 1;
    pub const PHY_REG_PHYID1: u64 = 2;
    pub const PHY_REG_PHYID2: u64 = 3;
    pub const PHY_REG_ANEGADV: u64 = 4;
    pub const PHY_REG_LINKPABIL: u64 = 5;
    pub const PHY_REG_ANEGEXP: u64 = 6;
    pub const PHY_REG_NEXTP: u64 = 7;
    pub const PHY_REG_LINKPNEXTP: u64 = 8;
    pub const PHY_REG_100BTCTRL: u64 = 9;
    pub const PHY_REG_1000BTSTAT: u64 = 10;
    pub const PHY_REG_EXTSTAT: u64 = 15;
    pub const PHY_REG_PHYSPCFC_CTL: u64 = 16;
    pub const PHY_REG_PHYSPCFC_ST: u64 = 17;
    pub const PHY_REG_INT_EN: u64 = 18;
    pub const PHY_REG_INT_ST: u64 = 19;
    pub const PHY_REG_EXT_PHYSPCFC_CTL: u64 = 20;
    pub const PHY_REG_RXERR: u64 = 21;
    pub const PHY_REG_EACD: u64 = 22;
    pub const PHY_REG_LED: u64 = 24;
    pub const PHY_REG_LED_OVRD: u64 = 25;
    pub const PHY_REG_EXT_PHYSPCFC_CTL2: u64 = 26;
    pub const PHY_REG_EXT_PHYSPCFC_ST: u64 = 27;
    pub const PHY_REG_CABLE_DIAG: u64 = 28;

    pub const PHY_REG_CONTROL_RST: u64 = 0x8000;
    pub const PHY_REG_CONTROL_LOOP: u64 = 0x4000;
    pub const PHY_REG_CONTROL_ANEG: u64 = 0x1000;

    pub const PHY_REG_STATUS_LINK: u64 = 0x0004;
    pub const PHY_REG_STATUS_ANEGCMPL: u64 = 0x0020;

    pub const PHY_REG_INT_ST_ANEGCMPL: u64 = 0x0800;
    pub const PHY_REG_INT_ST_LINKC: u64 = 0x0400;
    pub const PHY_REG_INT_ST_ENERGY: u64 = 0x0010;

    /***********************************************************************/
    pub const GEM_RX_REJECT: u64 = -1;
    pub const GEM_RX_PROMISCUOUS_ACCEPT: u64 = -2;
    pub const GEM_RX_BROADCAST_ACCEPT: u64 = -3;
    pub const GEM_RX_MULTICAST_HASH_ACCEPT: u64 = -4;
    pub const GEM_RX_UNICAST_HASH_ACCEPT: u64 = -5;

    pub const GEM_RX_SAR_ACCEPT: u64 = 0;

    /***********************************************************************/

    pub const DESC_1_USED: u64 = 0x80000000;
    pub const DESC_1_LENGTH: u64 = 0x00001FFF;

    pub const DESC_1_TX_WRAP: u64 = 0x40000000;
    pub const DESC_1_TX_LAST: u64 = 0x00008000;

    pub const DESC_0_RX_WRAP: u64 = 0x00000002;
    pub const DESC_0_RX_OWNERSHIP: u64 = 0x00000001;

    pub const R_DESC_1_RX_SAR_SHIFT: u64 = 25;
    pub const R_DESC_1_RX_SAR_LENGTH: u64 = 2;
    pub const R_DESC_1_RX_SAR_MATCH: u64 = 1 << 27;
    pub const R_DESC_1_RX_UNICAST_HASH: u64 = 1 << 29;
    pub const R_DESC_1_RX_MULTICAST_HASH: u64 = 1 << 30;
    pub const R_DESC_1_RX_BROADCAST: u64 = 1 << 31;

    pub const DESC_1_RX_SOF: u64 = 0x00004000;
    pub const DESC_1_RX_EOF: u64 = 0x00008000;

    pub const GEM_MODID_VALUE: u64 = 0x00020118;
}
pub use constants::*;

const GEM_DMACFG: u64 = 0x00000010;

const GEM_DMACFG_ADDR_64B: u32 = 1 << 30;

const VIRTIO_MTU: u16 = 2048;

#[repr(transparent)]
struct RxDesc([u32; 4]);
#[repr(transparent)]
struct TxDesc([u32; 4]);

/// Driver for the Cadence GEM Ethernet device.
pub struct MacbDriver {
    control_registers: MemoryRegion<u32>,
    mac: [u8; 6],

    rx_buffers: [[u8; 2048]; 8],
    rx_queue: [RxDesc; 8],
    tx_buffers: [[u8; 2048]; 8],
    tx_queue: [TxDesc; 8],
}

impl MacbDriver {
    pub const fn new(control_registers: MemoryRegion<u32>) -> Self {
        Self {
            control_registers,
            mac: [0; 6],

            rx_buffers: [[0; 2048]; 8],
            rx_queue: [RxDesc([0; 4]); 8],
            tx_buffers: [[0; 2048]; 8],
            tx_queue: [TxDesc([0; 4]); 8],
        }
    }

}

impl Driver for MacbDriver {
    const DEVICE_ID: u32 = 1;
    const FEATURES: u64 = VIRTIO_NET_F_MAC | VIRTIO_NET_F_MTU;
    const QUEUE_NUM_MAX: u32 = 2;

    fn interrupt(&mut self) -> bool {
        false
    }
    fn doorbell(&mut self, queue: u32) {

    }

    fn read_config_u8(&mut self, offset: u64) -> u8 {
        match offset {
            0..=5 => self.mac[offset as usize],
            10 => VIRTIO_MTU.to_le_bytes()[0],
            11 => VIRTIO_MTU.to_le_bytes()[1],
            _ => 0
        }
    }
    fn write_config_u8(&mut self, offset: u64, value: u8) {
        match offset {
            0..=5 => {
                self.mac[offset as usize] = value;
                unimplemented!(); // TODO: set device MAC to updated value
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.control_registers[GEM_DMACFG] = 0;
        self.control_registers[GEM_DMACFG] |= GEM_DMACFG_ADDR_64B;

        let rx_ptr = pmap::sa2pa(self.rx_queue.as_ptr() as u64);
        self.control_registers[GEM_RXQBASE] = rx_ptr as u32;
        self.control_registers[GEM_RBQPH] = (rx_ptr >> 32) as u32;

        let tx_ptr = pmap::sa2pa(self.rx_queue.as_ptr() as u64);
        self.control_registers[GEM_TXQBASE] = tx_ptr as u32;
        self.control_registers[GEM_TBQPH] = (tx_ptr >> 32) as u32;

        for (i, &mut desc) in self.rx_queue.iter_mut().enumerate() {
            let ptr = pmap::sa2pa(self.rx_buffers[i].as_raw());
            desc[0] = ptr as u32;
            desc[1]
        }
    }
}