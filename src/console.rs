use log::Level;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum ConsoleFunc {
    Assert = 1,
    Log = 2,
    Debug = 3,
    Trace = 4,
    Info = 5,
    Warn = 6,
    Error = 7,
    Exception = 8,
    Dir = 9,
}

impl ConsoleFunc {
    pub fn level(&self) -> Level {
        match *self {
            Self::Assert => Level::Error,
            Self::Log => Level::Debug,
            Self::Debug => Level::Debug,
            Self::Trace => Level::Trace,
            Self::Info => Level::Info,
            Self::Warn => Level::Warn,
            Self::Error => Level::Error,
            Self::Exception => Level::Error,
            Self::Dir => Level::Debug,
        }
    }
}

impl From<u32> for ConsoleFunc {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::Assert,
            2 => Self::Log,
            3 => Self::Debug,
            4 => Self::Trace,
            5 => Self::Info,
            6 => Self::Warn,
            7 => Self::Error,
            8 => Self::Exception,
            9 => Self::Dir,
            _ => Self::Log,
        }
    }
}
