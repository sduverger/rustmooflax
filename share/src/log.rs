
use uart::Serial;

pub struct Logger {
    output: Option<Serial>,
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        $crate::log::log_fmt(format_args!($($arg)*));
    });
}

// Public Write trait, provides write_fmt() but requires write_str()
use core::fmt;
use core::fmt::Write;

impl fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.output {
            Some(ref output) => output.write(s),
            None => ()
        }
        Ok(())
    }
}

#[cfg(feature = "setup")]
impl Logger {
    pub fn init(uart: Serial) {
        let mut logger = LOGGER.lock();

        match logger.output {
            None => logger.output = Some(uart),
            _ => ()
        }
    }
}

#[cfg(feature = "setup")]
use spin::Mutex;

#[cfg(feature = "setup")]
pub static LOGGER: Mutex<Logger> = Mutex::new(Logger { output : None });

#[cfg(feature = "setup")]
pub fn log_fmt(args: fmt::Arguments) {
    LOGGER.lock().write_fmt(args).unwrap();
}

#[cfg(feature = "vmm")]
pub fn log_fmt(args: fmt::Arguments) {
    use uart::SerialPort;

    let mut logger = Logger {
        output : Some(
            Serial {
                base: SerialPort::Com1 as u16
            }
        )};

    logger.write_fmt(args).unwrap();
}
