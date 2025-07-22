use serde::{Serialize};


#[derive(Serialize)]
pub struct JobInteractionStatus {
    pub applied: bool,
    pub saved: bool,
}