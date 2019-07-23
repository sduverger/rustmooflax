// Serial Port driver
use x86_64::instructions::port;

#[repr(u16)]
pub enum SerialPort {
    Com1 = 0x3f8,
    // Com2 = 0x2f8,
    // Com3 = 0x3e8,
    // Com4 = 0x2e8,
}

#[derive(Default)]
pub struct Serial {
    pub base : u16,
}


/*
** We use BitFlags wrapped into a module
** to have a namespace around bitfields
*/

// Enhanced Feature Register (EFR)
#[allow(non_snake_case)]
pub mod SerialEfr {
    bitflags! {
        pub struct Flags: u8 {
            const FLOW_CTL_TX1   = 0b10<<2;
            const FLOW_CTL_TX2   = 0b01<<2;
            const FLOW_CTL_RX1   = 0b10;
            const FLOW_CTL_RX2   = 0b01;
            const CTL            = 1<<4;
            const SPC            = 1<<5;
            const RTS            = 1<<6;
            const CTS            = 1<<7;
        }
    }
}

// Fifo Control Register (FCR)
#[allow(non_snake_case)]
pub mod SerialFcr {
    bitflags! {
        pub struct Flags: u8 {
            const ENABLE       = 1<<0;
            const RX           = 1<<1;
            const TX           = 1<<2;
            const DMA          = 1<<3;
        }
    }
}

// Line Control Register (LCR)
#[allow(non_snake_case)]
pub mod SerialLcr {
    bitflags! {
        pub struct Flags: u8 {
            const WDL_5        = 0b00;
            const WDL_6        = 0b01;
            const WDL_7        = 0b10;
            const WDL_8        = 0b11;
            const STOP         =     1 << 2;
            const PARITY_NONE  = 0b000 << 3;
            const PARITY_ODD   = 0b001 << 3;
            const PARITY_EVEN  = 0b011 << 3;
            const PARITY_MARK  = 0b101 << 3;
            const PARITY_SPACE = 0b111 << 3;
            const BRK          =     1 << 6;
            const DLA          =     1 << 7;
        }
    }
}

// Line Status Register (LSR)
#[allow(non_snake_case)]
pub mod SerialLsr {
    bitflags! {
        pub struct Flags: u8 {
            const DATA    = 0b00000001;
            const OVERRUN = 0b00000010;
            const PARITY  = 0b00000100;
            const FRAMING = 0b00001000;
            const BREAK   = 0b00010000;
            const THRE    = 0b00100000;
            const TSRE    = 0b01000000;
            const FIFO    = 0b10000000;
        }
    }
}

impl Serial {
    fn reg_lsr(&self)  -> u16 { self.base + 5 }
    fn reg_tx(&self)   -> u16 { self.base }

    // i/o port access (unsafe)
    fn write_reg(&self, reg: u16, val: u8) {
        unsafe { port::outb(reg, val) };
    }

    fn read_reg(&self, reg: u16) -> u8 {
        unsafe { port::inb(reg) }
    }

    fn can_send(&self) -> bool {
        let raw = self.read_reg(self.reg_lsr());
        let lsr = SerialLsr::Flags::from_bits_truncate(raw);

        lsr.contains(SerialLsr::THRE)
    }

    fn write_byte(&self, byte: u8) -> bool {
        if self.can_send() {
            self.write_reg(self.reg_tx(), byte);
            return true
        }

        false
    }

    pub fn write(&self, s: &str) {
        for byte in s.bytes() {
            while ! self.write_byte(byte) {}
        }
    }
}

#[cfg(feature = "setup")]
impl Serial {
    // register mapping functions
    fn reg_ier(&self)  -> u16 { self.base + 1 }
    fn reg_fcr(&self)  -> u16 { self.base + 2 }
    fn reg_lcr(&self)  -> u16 { self.base + 3 }
    // if lcr.dla = 1
    fn reg_dla_lsb(&self) -> u16 { self.base }
    fn reg_dla_msb(&self) -> u16 { self.base + 1 }
    // if lcr = 0xbf
    fn reg_efr(&self)  -> u16 { self.base + 2 }

    fn enable_efr_registers(&self) {
        self.write_reg(self.reg_lcr(), 0xbf);
    }

    fn enable_dla_registers(&self) {
        self.write_reg(self.reg_lcr(), 0x80);
    }

    // set baud rate to 115200
    fn set_dla_rate(&self) {
        self.write_reg(self.reg_dla_lsb(), 1);
        self.write_reg(self.reg_dla_msb(), 0);
    }

    fn fifo_init(&self) {
        let mut efr : SerialEfr::Flags = SerialEfr::CTL;

        self.enable_efr_registers();

        self.write_reg(self.reg_efr(), efr.bits());
        self.write_reg(self.reg_fcr(), 0);
        self.write_reg(self.reg_lcr(), 0);

        efr = SerialEfr::RTS | SerialEfr::CTS;
        self.write_reg(self.reg_efr(), efr.bits());
    }

    fn common_init(&self) {
        /* 8 bits length, 1 stop bit, no parity */
        let lcr : SerialLcr::Flags = SerialLcr::WDL_8;

        self.enable_dla_registers();
        self.set_dla_rate();
        self.write_reg(self.reg_lcr(), lcr.bits());
        self.write_reg(self.reg_ier(), 0);
    }

    fn full_init(&self) {
        self.fifo_init();
        self.common_init();
    }

    // associated function as constructor
    pub fn init(port: SerialPort) -> Serial {
        let uart = Serial { base: port as u16,
                            .. Default::default() };

        uart.full_init();
        uart
    }
}
