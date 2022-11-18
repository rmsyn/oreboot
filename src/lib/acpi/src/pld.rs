#[repr(C)]
pub enum PldPanel {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
    Unknown,
}

#[repr(C)]
pub enum PldVerticalPosition {
    Upper,
    Center,
    Lower,
}

/// The ACPI spec 6.2A does not define the horizontal position field.
/// These values are taken from the IASL compiler:
/// https://github.com/acpica/acpica/blob/master/source/components/utilities/utglobal.c#L321
#[repr(C)]
pub enum PldHorizontalPosition {
    Left,
    Center,
    Right,
}

#[repr(C)]
pub enum PldShape {
    Round,
    Oval,
    Square,
    VerticalRectangle,
    HorizontalRectangle,
    VerticalTrapezoid,
    HorizontalTrapezoid,
    Unknown,
    Chamfered,
}

#[repr(C)]
pub enum PldOrientation {
    Horizontal,
    Vertical,
}

#[repr(C)]
pub enum PldRotate {
    Rotate0,
    Rotate45,
    Rotate90,
    Rotate135,
    Rotate180,
    Rotate225,
    Rotate270,
    Rotate315,
}

#[repr(C)]
pub struct PldGroup {
    token: u8,
    position: u8,
}

#[repr(C)]
pub struct Pld {
    /* Color field can be explicitly ignored */
    ignore_color: bool,
    color_red: u8,
    color_blue: u8,
    color_green: u8,

    /* Port characteristics */
    /// Can be seen by the user
    visible: bool,
    /// Port is on lid of device
    lid: bool,
    /// Port is in a docking station
    dock: bool,
    /// Port is in a bay
    bay: bool,
    /// Device is ejectable, has _EJx objects
    ejectable: bool,
    /// Device needs OSPM to eject
    ejectable_ospm: bool,
    /// Width in mm
    width: u16,
    /// Height in mm
    height: u16,
    vertical_offset: u16,
    horizontal_offset: u16,
    panel: PldPanel,
    horizontal_position: PldHorizontalPosition,
    vertical_position: PldVerticalPosition,
    shape: PldShape,
    rotation: PldRotate,

    /* Port grouping */
    orientation: PldOrientation,
    group: PldGroup,
    draw_order: u8,
    cabinet_number: u8,
    card_cage_number: u8,

    /* Set if this PLD defines a reference shape */
    reference_shape: bool,
}

impl Pld {
    pub fn to_buffer(&self) -> [u8; 20] {
        let mut buf = [0u8; 20];

        /* [0] Revision (=2) */
        buf[0] = 0x2;

        if self.ignore_color {
            /* [1] Ignore Color */
            buf[0] |= 0x80;
        } else {
            /* [15:8] Red Color */
            buf[1] = self.color_red;
            /* [23:16] Green Color */
            buf[2] = self.color_green;
            /* [31:24] Blue Color */
            buf[3] = self.color_blue;
        }
        /* [47:32] Width */
        buf[4] = (self.width & 0xff) as u8;
        buf[5] = (self.width >> 8) as u8;

        /* [63:48] Height */
        buf[6] = (self.height & 0xff) as u8;
        buf[7] = (self.height >> 8) as u8;

        /* [64] User Visible */
        buf[8] |= self.visible as u8 & 0x1;

        /* [65] Dock */
        buf[8] |= (self.dock as u8 & 0x1) << 1;

        /* [66] Lid */
        buf[8] |= (self.lid as u8 & 0x1) << 2;

        /* [69:67] Panel */
        buf[8] |= (self.panel as u8 & 0x7) << 3;

        /* [71:70] Vertical Position */
        buf[8] |= (self.vertical_position as u8 & 0x3) << 6;

        /* [73:72] Horizontal Position */
        buf[9] |= self.horizontal_position as u8 & 0x3;

        /* [77:74] Shape */
        buf[9] |= (self.shape as u8 & 0xf) << 2;

        /* [78] Orientation */
        buf[9] |= (self.orientation as u8 & 0x1) << 6;

        /* [86:79] Group Token (incorrectly defined as 1 bit in ACPI 6.2A) */
        buf[9] |= (self.group.token & 0x1) << 7;
        buf[10] |= (self.group.token >> 0x1) & 0x7f;

        /* [94:87] Group Position */
        buf[10] |= (self.group.position & 0x1) << 7;
        buf[11] |= (self.group.position >> 0x1) & 0x7f;

        /* [95] Bay */
        buf[11] |= (self.bay as u8 & 0x1) << 7;

        /* [96] Ejectable */
        buf[12] |= self.ejectable as u8 & 0x1;

        /* [97] Ejectable with OSPM help */
        buf[12] |= (self.ejectable_ospm as u8 & 0x1) << 1;

        /* [105:98] Cabinet Number */
        buf[12] |= (self.cabinet_number & 0x3f) << 2;
        buf[13] |= (self.cabinet_number >> 6) & 0x3;

        /* [113:106] Card Cage Number */
        buf[13] |= (self.card_cage_number & 0x3f) << 2;
        buf[14] |= (self.card_cage_number >> 6) & 0x3;

        /* [114] PLD is a Reference Shape */
        buf[14] |= (self.reference_shape as u8 & 0x1) << 2;

        /* [118:115] Rotation */
        buf[14] |= (self.rotation as u8 & 0xf) << 3;

        /* [123:119] Draw Order */
        buf[14] |= (self.draw_order & 0x1) << 7;
        buf[15] |= (self.draw_order >> 1) & 0xf;

        /* [127:124] Reserved */

        /* Both 16 byte and 20 byte buffers are supported by the spec */
        /* FIXME: only 20 byte buffer supported in impl */
        if buf.len() == 20 {
            /* [143:128] Vertical Offset */
            buf[16] = (self.vertical_offset & 0xff) as u8;
            buf[17] = (self.vertical_offset >> 8) as u8;

            /* [159:144] Horizontal Offset */
            buf[18] = (self.horizontal_offset & 0xff) as u8;
            buf[19] = (self.horizontal_offset >> 8) as u8;
        }

        buf
    }
}
