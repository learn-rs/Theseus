//! Exception handlers that are task-aware, and will kill a task on an exception.

#![no_std]
#![feature(abi_x86_interrupt)]

extern crate x86_64;
extern crate task;
extern crate apic;
#[macro_use] extern crate vga_buffer; // for println_raw!()
#[macro_use] extern crate console; // for regular println!()


use x86_64::structures::idt::{LockedIdt, ExceptionStackFrame, PageFaultErrorCode};


pub fn init(idt_ref: &'static LockedIdt) {
    { 
        let mut idt = idt_ref.lock(); // withholds interrupts

        // SET UP FIXED EXCEPTION HANDLERS
        idt.divide_by_zero.set_handler_fn(divide_by_zero_handler);
        // missing: 0x01 debug exception
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // missing: 0x04 overflow exception
        // missing: 0x05 bound range exceeded exception
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        // reserved: 0x09 coprocessor segment overrun exception
        // missing: 0x0a invalid TSS exception
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        // missing: 0x0c stack segment exception
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        // reserved: 0x0f vector 15
        // missing: 0x10 floating point exception
        // missing: 0x11 alignment check exception
        // missing: 0x12 machine check exception
        // missing: 0x13 SIMD floating point exception
        // missing: 0x14 virtualization vector 20
        // missing: 0x15 - 0x1d SIMD floating point exception
        // missing: 0x1e security exception
        // reserved: 0x1f
    }

    idt_ref.load();
}


/// calls println!() and then println_raw!()
macro_rules! println_both {
    ($fmt:expr) => {
        print!(concat!($fmt, "\n"));
        print_raw!(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        print!(concat!($fmt, "\n"), $($arg)*);
        print_raw!(concat!($fmt, "\n"), $($arg)*);
    };
}


/// Kills the current task (the one that caused an exception)
/// and then halts that task (another task should be scheduled in).
fn kill_and_halt(exception_number: u8) -> ! {
    if let Some(taskref) = task::get_my_current_task() {
        taskref.write().kill(task::KillReason::Exception(exception_number));
    }

    loop { }
}



/// exception 0x00
pub extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: &mut ExceptionStackFrame) {
    println_both!("\nEXCEPTION: DIVIDE BY ZERO\n{:#?}\n", stack_frame);

    kill_and_halt(0x0)
}


/// exception 0x02, also used for TLB Shootdown IPIs
extern "x86-interrupt" fn nmi_handler(stack_frame: &mut ExceptionStackFrame) {
    // currently we're using NMIs to send TLB shootdown IPIs
    let vaddrs = apic::TLB_SHOOTDOWN_IPI_VIRTUAL_ADDRESSES.read();
    if !vaddrs.is_empty() {
        // trace!("nmi_handler (AP {})", apic::get_my_apic_id().unwrap_or(0xFF));
        apic::handle_tlb_shootdown_ipi(&vaddrs);
        return;
    }
    
    // if vaddr is 0, then it's a regular NMI    
    println_both!("\nEXCEPTION: NON-MASKABLE INTERRUPT at {:#x}\n{:#?}\n",
             stack_frame.instruction_pointer,
             stack_frame);
    
    kill_and_halt(0x2)
}


/// exception 0x03
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    println_both!("\nEXCEPTION: BREAKPOINT at {:#x}\n{:#?}\n",
             stack_frame.instruction_pointer,
             stack_frame);

    // don't halt here, this isn't a fatal/permanent failure, just a brief pause.
}

/// exception 0x06
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: &mut ExceptionStackFrame) {
    println_both!("\nEXCEPTION: INVALID OPCODE at {:#x}\n{:#?}\n",
             stack_frame.instruction_pointer,
             stack_frame);

    kill_and_halt(0x6)
}

/// exception 0x07
/// see this: http://wiki.osdev.org/I_Cant_Get_Interrupts_Working#I_keep_getting_an_IRQ7_for_no_apparent_reason
pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: &mut ExceptionStackFrame) {
    println_both!("\nEXCEPTION: DEVICE_NOT_AVAILABLE at {:#x}\n{:#?}\n",
             stack_frame.instruction_pointer,
             stack_frame);

    kill_and_halt(0x7)
}


pub extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut ExceptionStackFrame, _error_code: u64) {
    println_both!("\nEXCEPTION: DOUBLE FAULT\n{:#?}\n", stack_frame);
    
    kill_and_halt(0x8)
}


pub extern "x86-interrupt" fn segment_not_present_handler(stack_frame: &mut ExceptionStackFrame, error_code: u64) {
    println_both!("\nEXCEPTION: SEGMENT_NOT_PRESENT FAULT\nerror code: \
                                  {:#b}\n{:#?}\n",
             error_code,
             stack_frame);

    kill_and_halt(0xB)
}


pub extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: &mut ExceptionStackFrame, error_code: u64) {
    println_both!("\nEXCEPTION: GENERAL PROTECTION FAULT \nerror code: \
                                  {:#X}\n{:#?}\n",
             error_code,
             stack_frame);

    kill_and_halt(0xD)
}


pub extern "x86-interrupt" fn page_fault_handler(stack_frame: &mut ExceptionStackFrame, error_code: PageFaultErrorCode) {
    use x86_64::registers::control_regs;
    println_both!("\nEXCEPTION: PAGE FAULT while accessing {:#x}\nerror code: \
                                  {:?}\n{:#?}\n",
             control_regs::cr2(),
             error_code,
             stack_frame);
    
    kill_and_halt(0xE)
}