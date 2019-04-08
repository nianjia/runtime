mod posix;
#[cfg(target_os = "linux")] 
pub use self::posix::*;