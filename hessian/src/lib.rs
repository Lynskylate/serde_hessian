mod constant;
pub mod de;
mod error;
pub mod ser;
pub mod value;

pub use constant::ByteCodecType;
pub use de::from_slice;
pub use error::{Error, ErrorKind};
pub use ser::to_vec;
pub use value::Value;
