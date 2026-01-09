/// The code in this module is inspired from osdev
/// 8259_PIC guide.
use common::enums::{
    CascadedPicInterruptLine, PicCommandCode, PicInterruptLine,
    PicInterruptVectorOffset, PicMode, Port,
};
use cpu_utils::instructions::port::PortExt;

pub static mut PIC: CascadedPIC = CascadedPIC::default();

struct ProgrammableInterruptController {
    command: Port,
    data: Port,
    interrupt_offset: PicInterruptVectorOffset,
}

impl ProgrammableInterruptController {
    const fn default_master() -> Self {
        Self {
            command: Port::MasterPicCmd,
            data: Port::MasterPicData,
            interrupt_offset: PicInterruptVectorOffset::Master,
        }
    }
    const fn default_slave() -> Self {
        Self {
            command: Port::SlavePicCmd,
            data: Port::SlavePicData,
            interrupt_offset: PicInterruptVectorOffset::Slave,
        }
    }

    fn disable_irq(&mut self, irq: PicInterruptLine) {
        unsafe {
            let prev_mask = self.data.inb();
            let new_mask = prev_mask | irq as u8;
            self.data.outb(new_mask);
        }
    }

    fn enable_irq(&mut self, irq: PicInterruptLine) {
        unsafe {
            let prev_mask = self.data.inb();
            let new_mask = prev_mask & !(irq as u8);
            self.data.outb(new_mask);
        }
    }

    fn enable(&mut self) {
        unsafe {
            self.data.outb(0);
        }
    }

    fn disable(&mut self) {
        unsafe {
            self.data.outb(0xff);
        }
    }

    fn end_of_interrupt(&mut self) {
        unsafe {
            self.command.outb(PicCommandCode::EndOfInterrupt as u8);
        }
    }
}

pub struct CascadedPIC {
    master: ProgrammableInterruptController,
    slave: ProgrammableInterruptController,
}

impl CascadedPIC {
    pub const fn default() -> Self {
        Self {
            master: ProgrammableInterruptController::default_master(),
            slave: ProgrammableInterruptController::default_slave(),
        }
    }

    pub fn init(uninit: &'static mut Self) {
        unsafe {
            // Send initialize command to master
            uninit.master.command.outb(
                PicCommandCode::Initialize as u8
                    | PicCommandCode::CascadeMode as u8,
            );
            Port::iowait();
            // Send initialize command to slave
            uninit.slave.command.outb(
                PicCommandCode::Initialize as u8
                    | PicCommandCode::CascadeMode as u8,
            );
            Port::iowait();
            // Send IVT offset to master
            uninit
                .master
                .data
                .outb(uninit.master.interrupt_offset as u8);
            Port::iowait();
            // Send IVT offset to slave
            uninit.slave.data.outb(uninit.slave.interrupt_offset as u8);
            Port::iowait();
            // Tell master how it is connected to slave
            uninit.master.data.outb(PicInterruptLine::Irq2 as u8);
            Port::iowait();
            // Tell slave how it is connected to master
            uninit.slave.data.outb(PicInterruptLine::Irq1 as u8);
            Port::iowait();
            // Set PIC mode of master
            uninit.master.data.outb(PicMode::Mode8086 as u8);
            Port::iowait();
            // Set PIC mode of slave
            uninit.slave.data.outb(PicMode::Mode8086 as u8);
            Port::iowait();
            uninit.master.enable();
            uninit.slave.enable();
        }
    }

    pub fn disable_irq(&mut self, irq: CascadedPicInterruptLine) {
        unsafe {
            if irq as u16 > PicInterruptLine::Irq7 as u16 {
                let irq: PicInterruptLine =
                    core::mem::transmute(((irq as u16) >> u8::BITS) as u8);
                self.slave.disable_irq(irq);
            } else {
                let irq: PicInterruptLine =
                    core::mem::transmute(irq as u8);
                self.master.disable_irq(irq);
            }
        }
    }
    pub fn enable_irq(&mut self, irq: CascadedPicInterruptLine) {
        unsafe {
            if irq as u16 >= CascadedPicInterruptLine::Irq8 as u16 {
                let irq: PicInterruptLine =
                    core::mem::transmute(((irq as u16) >> u8::BITS) as u8);
                self.slave.enable_irq(irq);
            } else {
                let irq: PicInterruptLine =
                    core::mem::transmute(irq as u8);
                self.master.enable_irq(irq);
            }
        }
    }

    pub fn end_of_interrupt(&mut self, irq: CascadedPicInterruptLine) {
        if irq as u16 >= CascadedPicInterruptLine::Irq8 as u16 {
            self.slave.end_of_interrupt();
        }
        self.master.end_of_interrupt();
    }
}
