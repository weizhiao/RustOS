#![no_std]
const INTR_PRIORITY_BASE: usize = 0x000000;
const INTR_ENABLE_BASE: usize = 0x002000;
const PRIORITY_THRESHOLD_BASE: usize = 0x200000;
const INTR_CLAIM_BASE: usize = 0x200004;

pub struct PLIC {
    base_addr: usize,
}

#[derive(Copy, Clone)]
pub enum TargetPriority {
    Machine = 0,
    Supervisor = 1,
}

impl TargetPriority {
    pub fn supported_number() -> usize {
        2
    }
}

impl PLIC {
    fn intr_priority_ptr(&self, intr_id: usize) -> *mut u32 {
        (self.base_addr + INTR_PRIORITY_BASE + intr_id * 4) as *mut u32
    }
    fn intr_target(hart_id: usize, target_priority: TargetPriority) -> usize {
        let priority_num = TargetPriority::supported_number();
        hart_id * priority_num + target_priority as usize
    }
    fn intr_enable_ptr(
        &self,
        hart_id: usize,
        target_priority: TargetPriority,
        intr_id: usize,
    ) -> (*mut u32, usize) {
        let id = Self::intr_target(hart_id, target_priority);
        let (reg_id, reg_shift) = (intr_id / 32, intr_id % 32);
        (
            (self.base_addr + INTR_ENABLE_BASE + 0x80 * id + 0x4 * reg_id) as *mut u32,
            reg_shift,
        )
    }
    fn priority_threshold_ptr(&self, hart_id: usize, target_priority: TargetPriority) -> *mut u32 {
        let id = Self::intr_target(hart_id, target_priority);
        (self.base_addr + PRIORITY_THRESHOLD_BASE + 0x1000 * id) as *mut u32
    }
    fn intr_claim_ptr(&self, hart_id: usize, target_priority: TargetPriority) -> *mut u32 {
        let id = Self::intr_target(hart_id, target_priority);
        (self.base_addr + INTR_CLAIM_BASE + 0x1000 * id) as *mut u32
    }
    pub unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }
    /// The interrupt priority for each interrupt source.
    pub fn set_intr_priority(&mut self, intr_id: usize, priority: u32) {
        unsafe {
            self.intr_priority_ptr(intr_id).write_volatile(priority);
        }
    }
    /// The interrupt priority for each interrupt source.
    pub fn get_intr_priority(&mut self, intr_id: usize) -> u32 {
        unsafe { self.intr_priority_ptr(intr_id).read_volatile() }
    }
    /// The enablement of interrupt source of each context.
    pub fn set_intr_enable(
        &mut self,
        hart_id: usize,
        target_priority: TargetPriority,
        intr_id: usize,
    ) {
        let (ptr, shift) = self.intr_enable_ptr(hart_id, target_priority, intr_id);
        unsafe {
            ptr.write_volatile(ptr.read_volatile() | 1u32 << shift);
        }
    }
    /// The enablement of interrupt source of each context.
    pub fn set_intr_disable(
        &mut self,
        hart_id: usize,
        target_priority: TargetPriority,
        intr_id: usize,
    ) {
        let (ptr, shift) = self.intr_enable_ptr(hart_id, target_priority, intr_id);
        unsafe {
            ptr.write_volatile(ptr.read_volatile() & (!(1u32 << shift)));
        }
    }
    /// The interrupt priority threshold of each context.
    pub fn set_target_threshold(
        &mut self,
        hart_id: usize,
        target_priority: TargetPriority,
        threshold: u32,
    ) {
        let threshold_ptr = self.priority_threshold_ptr(hart_id, target_priority);
        unsafe {
            threshold_ptr.write_volatile(threshold);
        }
    }
    /// The interrupt priority threshold of each context.
    pub fn get_target_threshold(&mut self, hart_id: usize, target_priority: TargetPriority) -> u32 {
        let threshold_ptr = self.priority_threshold_ptr(hart_id, target_priority);
        unsafe { threshold_ptr.read_volatile() }
    }
    /// The register to acquire interrupt source ID of each context.
    pub fn claim(&mut self, hart_id: usize, target_priority: TargetPriority) -> u32 {
        let claim_ptr = self.intr_claim_ptr(hart_id, target_priority);
        unsafe { claim_ptr.read_volatile() }
    }
    /// The register to send interrupt completion message to the associated gateway.
    pub fn complete(&mut self, hart_id: usize, target_priority: TargetPriority, intr_id: u32) {
        let comp_ptr = self.intr_claim_ptr(hart_id, target_priority);
        unsafe {
            comp_ptr.write_volatile(intr_id);
        }
    }
}
