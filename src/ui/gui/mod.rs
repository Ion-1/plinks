use crate::app::CommandArguments;
use crate::profile_loader::Installation;
use qmetaobject::prelude::*;

pub fn open_dialog(
    uri: String,
    installations: Vec<&mut Installation>,
) -> Option<CommandArguments> {
    qmetaobject::log::init_qt_to_rust();
    todo!()
}
