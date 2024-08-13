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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use crate::{ConsoleFunc, DukContext, JsEngine, JsError, JsInterop, Return};

    #[derive(Clone, Debug)]
    struct ConsoleInterop {
        messages: Arc<Mutex<Vec<String>>>
    }

    impl JsInterop for ConsoleInterop {
        fn call(&mut self, _engine: &mut DukContext, _func_name: &str) -> Result<Return, JsError> {
            Ok(Return::Error)
        }

        fn console(&mut self, func: ConsoleFunc, msg: &str) {
            self.messages.lock().unwrap().push(format!("{}: {}", func.level(), msg));
        }
    }

    #[test]
    fn test_console() {
        let interop = ConsoleInterop {
            messages: Arc::new(Mutex::new(Vec::new()))
        };
        let engine = JsEngine::with_interop(interop.clone()).unwrap();
        engine.init_console();

        engine.eval("console.log('test message')").unwrap();

        let messages = interop.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "DEBUG: test message");
    }

    #[test]
    fn test_console_in_new_globalenv() {
        let interop = ConsoleInterop {
            messages: Arc::new(Mutex::new(Vec::new()))
        };
        let engine = JsEngine::with_interop(interop.clone()).unwrap();

        let idx = engine.push_thread_new_globalenv();
        let ctx = engine.get_context(idx).unwrap();
        ctx.init_console();

        ctx.eval("console.log('test message')").unwrap();

        let messages = interop.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "DEBUG: test message");
    }
}
