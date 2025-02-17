use bitdemon::networking::bd_session::SessionId;
use bitdemon::networking::session_manager::SessionManager;
use env_logger::fmt::{style, Formatter};
use log::{LevelFilter, Record};
use std::cell::Cell;
use std::fmt::Display;
use std::io;
use std::io::Write;

pub fn initialize_log() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .format(move |buf, record| {
            let fmt = CustomFormat {
                written_header_value: false,
                buf,
            };

            fmt.write(record)
        })
        .init();
}

#[derive(Copy, Clone)]
struct SessionLogData {
    pub session_id: SessionId,
    pub session_type_name: &'static str,
}

impl Display for SessionLogData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.session_type_name, self.session_id)
    }
}

thread_local! {
    static SESSION_LOG_DATA: Cell<Option<SessionLogData>> = const { Cell::new(None) };
}

pub fn log_session_id(session_manager: &SessionManager, session_type_name: &'static str) {
    session_manager.on_session_registered(move |session| {
        SESSION_LOG_DATA.set(Some(SessionLogData {
            session_id: session.id,
            session_type_name,
        }))
    })
}

struct CustomFormat<'a> {
    written_header_value: bool,
    buf: &'a mut Formatter,
}

// Based on Env Logger DefaultFormat
impl CustomFormat<'_> {
    fn write(mut self, record: &Record<'_>) -> io::Result<()> {
        self.write_timestamp()?;
        self.write_level(record)?;
        self.write_target(record)?;
        self.write_session()?;
        self.finish_header()?;

        self.write_args(record)?;
        writeln!(self.buf)
    }

    fn write_header_value<T>(&mut self, value: T) -> io::Result<()>
    where
        T: Display,
    {
        if !self.written_header_value {
            self.written_header_value = true;
            let style = style::AnsiColor::BrightBlack.on_default();
            write!(self.buf, "{style}[{style:#}{value}")
        } else {
            write!(self.buf, " {value}")
        }
    }

    fn write_level(&mut self, record: &Record<'_>) -> io::Result<()> {
        let level = record.level();
        let level_style = self.buf.default_level_style(level);

        self.write_header_value(format_args!("{level_style}{level:<5}{level_style:#}"))
    }

    fn write_timestamp(&mut self) -> io::Result<()> {
        self.write_header_value(self.buf.timestamp_millis())
    }

    fn write_target(&mut self, record: &Record<'_>) -> io::Result<()> {
        match record.target() {
            "" => Ok(()),
            target => self.write_header_value(target),
        }
    }

    fn write_session(&mut self) -> io::Result<()> {
        if let Some(session_log_data) = SESSION_LOG_DATA.get() {
            self.write_header_value(session_log_data)
        } else {
            Ok(())
        }
    }

    fn finish_header(&mut self) -> io::Result<()> {
        if self.written_header_value {
            let style = style::AnsiColor::BrightBlack.on_default();
            write!(self.buf, "{style}]{style:#} ")
        } else {
            Ok(())
        }
    }

    fn write_args(&mut self, record: &Record<'_>) -> io::Result<()> {
        write!(self.buf, "{}", record.args())
    }
}
