#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClassId {
	/* These are used internally by driver model */
	Root = 0,
	Demo,
	Test,
	TestFdt,
	TestFdtManual,
	TestBUS,
	TestProbe,
	TestDummy,
	TestDevres,
	TestAcpi,
	SpiEmul,	/* sandbox SPI device emulator */
	I2cEmul,	/* sandbox I2C device emulator */
	I2cEmulParent,	/* parent for I2C device emulators */
	PciEmul,	/* sandbox PCI device emulator */
	PciEmulParent,	/* parent for PCI device emulators */
	UsbEmul,	/* sandbox USB bus device emulator */
	AxiEmul,	/* sandbox AXI bus device emulator */

	/* U-Boot uclasses start here - in alphabetical order */
	AcpiPmc,	/* (x86) Power-management controller (PMC) */
	Adc,		/* Analog-to-digital converter */
	Ahci,		/* SATA disk controller */
	AudioCodec,	/* Audio codec with control and data path */
	Axi,		/* AXI bus */
	Blk,		/* Block device */
	Bootcount,       /* Bootcount backing store */
	Bootdev,		/* Boot device for locating an OS to boot */
	Bootmeth,	/* Bootmethod for booting an OS */
	Bootstd,		/* Standard boot driver */
	Button,		/* Button */
	Cache,		/* Cache controller */
	Clk,		/* Clock source, e.g. used by peripherals */
	Cpu,		/* CPU, typically part of an SoC */
	CrosUc,		/* Chrome OS EC */
	Display,		/* Display (e.g. DisplayPort, HDMI) */
	Dma,		/* Direct Memory Access */
	Dsa,		/* Distributed (Ethernet) Switch Architecture */
	DsiHost,	/* Display Serial Interface host */
    Ecdsa,		/* Elliptic curve cryptographic device */
	EfiLoader,	/* Devices created by UEFI applications */
	EfiMedia,	/* Devices provided by UEFI firmware */
    Eth,		/* Ethernet device */
	EthPhy,		/* Ethernet PHY device */
    Firmware,	/* Firmware */
    Fpga,		/* FPGA device */
	FuzzingEngine,	/* Fuzzing engine */
	FsFirmwareLoader,		/* Generic loader */
	FwuMdata,	/* FWU Metadata Access */
    Gpio,		/* Bank of general-purpose I/O pins */
    Hash,		/* Hash device */
    Hwspinlock,	/* Hardware semaphores */
    Host,		/* Sandbox host device */
	I2c,		/* I2C bus */
	I2cEeprom,	/* I2C EEPROM device */
	I2cGeneric,	/* Generic I2C device */
	I2cMux,		/* I2C multiplexer */
	I2s,		/* I2S bus */
    Ide,		/* IDE device */
    Iommu,		/* IOMMU */
    Irq,		/* Interrupt controller */
    Keyboard,	/* Keyboard input device */
    Led,		/* Light-emitting diode (LED) */
    Lpc,		/* x86 'low pin count' interface */
    Mailbox,		/* Mailbox controller */
	MassStorage,	/* Mass storage device */
    Mdio,		/* MDIO bus */
	MdioMux,	/* MDIO MUX/switch */
    Memory,		/* Memory Controller device */
    Misc,		/* Miscellaneous device */
    Mmc,		/* SD / MMC card or chip */
	ModExp,		/* RSA Mod Exp device */
    Mtd,		/* Memory Technology Device (MTD) device */
    Mux,		/* Multiplexer device */
    Nop,		/* No-op devices */
    Northbridge,	/* Intel Northbridge / SDRAM controller */
    Nvme,		/* NVM Express device */
	P2sb,		/* (x86) Primary-to-Sideband Bus */
    Panel,		/* Display panel, such as an LCD */
	PanelBacklight,	/* Backlight controller for panel */
    Partition,	/* Logical disk partition device */
    Pch,		/* x86 platform controller hub */
    Pci,		/* PCI bus */
	PciEp,		/* PCI endpoint device */
	PciGeneric,	/* Generic PCI bus device */
    Phy,		/* Physical Layer (PHY) device */
    Pinconfig,	/* Pin configuration node device */
    Pinctrl,		/* Pinctrl (pin muxing/configuration) device */
    Pmic,		/* PMIC I/O device */
	PowerDomain,	/* (SoC) Power domains */
    Pvblock,		/* Xen virtual block device */
    Pwm,		/* Pulse-width modulator */
    Pwrseq,		/* Power sequence device */
    Qfw,		/* QEMU firmware config device */
    Ram,		/* RAM controller */
	RebootMode,	/* Reboot mode */
    Regulator,	/* Regulator device */
    Remoteproc,	/* Remote Processor device */
    Reset,		/* Reset controller device */
    Rng,		/* Random Number Generator */
    Rtc,		/* Real time clock device */
	ScmiAgent,	/* Interface with an SCMI server */
    Scsi,		/* SCSI device */
    Serial,		/* Serial UART */
	SimpleBus,	/* Bus with child devices */
    Smem,		/* Shared memory interface */
    Soc,		/* SOC Device */
    Sound,		/* Playing simple sounds */
    Spi,		/* SPI bus */
	SpiFlash,	/* SPI flash */
	SpiGeneric,	/* Generic SPI flash target */
    Spmi,		/* System Power Management Interface bus */
    Syscon,		/* System configuration device */
    Sysinfo,		/* Device information from hardware */
    Sysreset,	/* System reset device */
    Tee,		/* Trusted Execution Environment device */
    Thermal,		/* Thermal sensor */
    Timer,		/* Timer device */
    Tpm,		/* Trusted Platform Module TIS interface */
    Ufs,		/* Universal Flash Storage */
    Usb,		/* USB bus */
	UsbDevGeneric,	/* USB generic device */
	UsbHub,		/* USB hub */
	UsbGadgetGeneric,	/* USB generic device */
    Video,		/* Video or LCD device */
	VideoBridge,	/* Video bridge, e.g. DisplayPort to LVDS */
	VideoConsole,	/* Text console driver for video device */
	VideoOsd,	/* On-screen display */
    Virtio,		/* VirtIO transport device */
	W1,		/* Dallas 1-Wire bus */
	W1Eeprom,	/* one-wire EEPROMs */
    Wdt,		/* Watchdog Timer driver */
    Count,
	Invalid = -1,
}
