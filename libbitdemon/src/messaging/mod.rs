mod bd_data_type;
pub mod bd_error_code;
pub mod bd_message;
pub mod bd_reader;
pub mod bd_writer;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum StreamMode {
    ByteMode,
    BitMode,
}
