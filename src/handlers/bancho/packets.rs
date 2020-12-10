use crate::{constants::id, packets::HandlerData};

impl id {
    pub async fn handle<'a>(&self, handler_data: &HandlerData<'a>, payload: Option<&[u8]>) {
        match self {
            id::OSU_PING => {}
            _ => {
                println!("unhandled: {:?}", self);
            }
        };
    }
}
