use spin::rwlock::RwLock;

static PACC: RwLock<PciAccess> = RwLock::new(PciAccess::new());

pub fn pci_write_config16(dev: &Device, where_: u32, val: u16) -> u16 {
    if let Ok(d) = pci_get_dev(*PACC.read(), 0, dev.busno, dev.slot, dev.func) {
        return pci_read_word(d, where_);
    }

    if cfg!(debug_pci) {
        debug!("PCI: device not found while read word ({:x}:{:x}:{:x})", dev.busno, dev.slot, dev.func);
    }

    0
}
