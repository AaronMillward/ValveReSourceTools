pub mod vpk;
pub mod mdl;

pub mod error {
	use std::fmt::Display;

	#[derive(Debug)]
	pub enum ErrorKind {
		InvalidHeader(String),
		MalformedData(String),
		DoesNotExist(String),
		AlreadyExists(String),
		ValidationFailed(String),
		
		/* Wrapped errors from other libs */
		IO(std::io::Error),
		Bincode(Box<bincode::ErrorKind>),
	}

	impl Display for ErrorKind {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match self {
				ErrorKind::IO(e)        => { e.fmt(f) },
				ErrorKind::InvalidHeader(e)    => write!(f, "Invalid Header: {}", e),
				ErrorKind::MalformedData(e)    => write!(f, "Malformed Data: {}", e),
				ErrorKind::DoesNotExist(e)     => write!(f, "Does Not Exist: {}", e),
				ErrorKind::AlreadyExists(e)    => write!(f, "Already Exists: {}", e),
				ErrorKind::ValidationFailed(e) => write!(f, "Validation Failed: {}", e),
				ErrorKind::Bincode(e)          => write!(f, "Bincode: {}", e),
			}
		}
	}

	impl std::error::Error for ErrorKind {}
	
	impl From<std::io::Error> for ErrorKind {
		fn from(e : std::io::Error) -> Self {
			ErrorKind::IO(e)
		}
	}

	impl From<Box<bincode::ErrorKind>> for ErrorKind {
		fn from(e: Box<bincode::ErrorKind>) -> Self {
			match *e {
				bincode::ErrorKind::Io(e) => ErrorKind::IO(e),
				_ => ErrorKind::Bincode(e),
			}
		}
	}
}

/// Transmutes a byte slice to `T`
/// 
/// # Arguments
/// * `data` - A byte slice containing the data to be transmuted
/// 
/// # Panics
/// * if `data` does not match the size of `T`
fn read_from_bytes<T>(data : &[u8]) -> T {
	assert!(data.len() == std::mem::size_of::<T>(), "Use of read_from_bytes without proper sizing");
	unsafe { std::ptr::read(data.as_ptr() as *const T) }
}