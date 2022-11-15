use super::lib::util::timer::Stopwatch;
use crate::ec::google::chromeec::ec::google_chromeec_get_board_version;

pub const BOARD_ID_UNKNOWN: u32 = ~0;  // unsigned equivalent to -1
pub const BOARD_ID_INIT: u32 = ~1;     // unsigned equivalent to -2

pub fn board_id(sw: &mut Stopwatch) -> u32 {
    let mut id = BOARD_ID_INIT;
    if id == BOARD_ID_INIT {
        if google_chromeec_get_board_version(&mut id, sw) {
            id = BOARD_ID_UNKNOWN;
        }
    }
    id
}
