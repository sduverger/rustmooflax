use info::data::InformationData;

// we force it 'mut' else it is optimized as a constant
// anywhere it is used (ie. get_info_data())
#[link_section = ".info_hdr"]
pub static mut INFO: u64 = 0;

pub fn info_data() -> &'static mut InformationData {
    unsafe{ &mut *(INFO as *mut _) }
}
