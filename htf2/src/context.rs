use crate::measurement::Measurements;
use user_text_input::TextInput;

pub mod user_text_input;

pub struct SysContext {
    pub text_input: TextInput,
    pub measurements: Measurements,
}
