use core::fmt;

use shared::PrivilegeLevel;

/// Specifies which element to load into a segment from
/// descriptor tables (i.e., is a index to LDT or GDT table
/// with some additional flags).
bitflags! {
    #[repr(C, packed)]
    pub flags SegmentSelector: u16 {
        /// Requestor Privilege Level
        const RPL_0 = 0b00,
        const RPL_1 = 0b01,
        const RPL_2 = 0b10,
        const RPL_3 = 0b11,

        /// Table Indicator (TI) 0 means GDT is used.
        const TI_GDT = 0 << 3,
        /// Table Indicator (TI) 1 means LDT is used.
        const TI_LDT = 1 << 3,
    }
}

/// Reload code segment register.
/// Note this is special since we can not directly move
/// to %cs. Instead we push the new segment selector
/// and return value on the stack and use lretq
/// to reload cs and continue at 1:.
pub unsafe fn set_cs(sel: SegmentSelector) {

    #[cfg(target_arch="x86")]
    #[inline(always)]
    unsafe fn inner(sel: SegmentSelector) {
        asm!("pushl $0; \
              pushl $$1f; \
              lretl; \
              1:" :: "ri" (sel.bits() as usize) : "{rax}" "memory");
    }

    #[cfg(target_arch="x86_64")]
    #[inline(always)]
    unsafe fn inner(sel: SegmentSelector) {
        asm!("pushq $0; \
              leaq  1f(%rip), %rax; \
              pushq %rax; \
              lretq; \
              1:" :: "ri" (sel.bits() as usize) : "{rax}" "memory");
    }

    inner(sel)
}


impl SegmentSelector {
    /// Create a new SegmentSelector
    ///
    /// # Arguments
    ///  * `index` index in GDT or LDT array.
    ///
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector { bits: index << 3 | (rpl as u16) }
    }

    pub const fn from_raw(bits: u16) -> SegmentSelector {
        SegmentSelector { bits: bits }
    }
}

impl fmt::Display for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let r0 = match self.contains(RPL_0) {
            false => "",
            true => "Ring 0 segment selector.",
        };
        let r1 = match self.contains(RPL_1) {
            false => "",
            true => "Ring 1 segment selector.",
        };
        let r2 = match self.contains(RPL_2) {
            false => "",
            true => "Ring 2 segment selector.",
        };
        let r3 = match self.contains(RPL_3) {
            false => "",
            true => "Ring 3 segment selector.",
        };
        let tbl = match self.contains(TI_LDT) {
            false => "GDT Table",
            true => "LDT Table",
        };

        write!(f,
               "Index {} in {}, {}{}{}{}",
               self.bits >> 3,
               tbl,
               r0,
               r1,
               r2,
               r3)
        // write!(f, "Index")
    }
}


/// Reload stack segment register.
pub unsafe fn load_ss(sel: SegmentSelector) {
    asm!("movw $0, %ss " :: "r" (sel) : "memory");
}

/// Reload data segment register.
pub unsafe fn load_ds(sel: SegmentSelector) {
    asm!("movw $0, %ds " :: "r" (sel) : "memory");
}

/// Reload es segment register.
pub unsafe fn load_es(sel: SegmentSelector) {
    asm!("movw $0, %es " :: "r" (sel) : "memory");
}

/// Reload fs segment register.
pub unsafe fn load_fs(sel: SegmentSelector) {
    asm!("movw $0, %fs " :: "r" (sel) : "memory");
}

/// Reload gs segment register.
pub unsafe fn load_gs(sel: SegmentSelector) {
    asm!("movw $0, %gs " :: "r" (sel) : "memory");
}

/// Returns the current value of the code segment register.
pub fn cs() -> SegmentSelector {
    let segment: u16;
    unsafe { asm!("mov %cs, $0" : "=r" (segment) ) };
    SegmentSelector::from_raw(segment)
}
