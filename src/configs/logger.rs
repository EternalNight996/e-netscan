use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// init log
pub fn init_logger(name: &str, verbose_level: u8) {
    let path = Path::new(".").join(format!("{}.yaml", name));
    match log4rs::init_file(path.clone(), Default::default()) {
        Ok(_) => {}
        Err(_e) => {
            match LogRoot::new(name).write() {
                Ok(p) => match log4rs::init_file(p, Default::default()) {
                    Ok(_) => {}
                    Err(e) => out(&e.to_string(), Some((255, 0, 0))),
                },
                Err(e) => out(&e, Some((255, 0, 0))),
            };
        }
    }
    log::set_max_level(match verbose_level {
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        5 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Off,
    });
}

struct LogRoot {
    path: PathBuf,
    appenders: LogAppenders,
}
impl LogRoot {
    fn new(name: &str) -> Self {
        LogRoot {
            path: Path::new(".").join(format!("{}.yaml", name)),
            appenders: LogAppenders {
                stdout: LogStdout {
                    kind: "console".to_owned(),
                    encoder: LogEncoder {
                        pattern: "[Console] {d} - {l} -{t} - {m}{n}".to_owned(),
                    },
                },
                file: LogFile {
                    kind: "rolling_file".to_owned(),
                    path: format!("logs/{}/requests.log", name),
                    encoder: LogEncoder {
                        pattern: "[File] {d} - {l} - {t} - {m}{n}".to_owned(),
                    },
                    policy: LogPolicy {
                        kind: "compound".to_owned(),
                        trigger: LogTrigger {
                            kind: "size".to_owned(),
                            limit: 5,
                        },
                        roller: LogRoller {
                            kind: "fixed_window".to_owned(),
                            pattern: format!("logs/{}/requests_{{}}.log", name),
                            base: 1,
                            count: 100,
                        },
                    },
                },
            },
        }
    }
    fn write(&self) -> Result<PathBuf, String> {
        match fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)
        {
            Ok(mut f) => {
                let s = self.to_string();
                if let Err(e) = f.write_all(s.as_bytes()) {
                    return Err(e.to_string());
                };
                if let Err(e) = f.sync_all() {
                    return Err(e.to_string());
                };
                Ok(self.path.clone())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
impl ToString for LogRoot {
    fn to_string(&self) -> String {
        let ref stdout = self.appenders.stdout;
        let stdout_s = format!(
            "\x20\x20stdout:
\x20\x20\x20\x20kind: {}
\x20\x20\x20\x20encoder:
\x20\x20\x20\x20\x20\x20pattern: \"{}\"\n",
            stdout.kind, stdout.encoder.pattern
        );
        let ref f = self.appenders.file;
        let file_s = format!(
            "\x20\x20file:
\x20\x20\x20\x20kind: {}
\x20\x20\x20\x20path: \"{}\"
\x20\x20\x20\x20encoder:
\x20\x20\x20\x20\x20\x20pattern: \"{}\"
\x20\x20\x20\x20policy:
\x20\x20\x20\x20\x20\x20kind: {}
\x20\x20\x20\x20\x20\x20trigger:
\x20\x20\x20\x20\x20\x20\x20\x20kind: {}
\x20\x20\x20\x20\x20\x20\x20\x20limit: {} mb
\x20\x20\x20\x20\x20\x20roller:
\x20\x20\x20\x20\x20\x20\x20\x20kind: {}
\x20\x20\x20\x20\x20\x20\x20\x20pattern: \"{}\"
\x20\x20\x20\x20\x20\x20\x20\x20base: {}
\x20\x20\x20\x20\x20\x20\x20\x20count: {}\n",
            f.kind,
            f.path,
            f.encoder.pattern,
            f.policy.kind,
            f.policy.trigger.kind,
            f.policy.trigger.limit,
            f.policy.roller.kind,
            f.policy.roller.pattern,
            f.policy.roller.base,
            f.policy.roller.count
        );
        let root_s = "root:
\x20\x20appenders:
\x20\x20\x20\x20- stdout
\x20\x20\x20\x20- file";
        format!("appenders:\n{}{}{}", stdout_s, file_s, root_s)
    }
}

struct LogAppenders {
    stdout: LogStdout,
    file: LogFile,
}

struct LogFile {
    kind: String,
    path: String,
    encoder: LogEncoder,
    policy: LogPolicy,
}

struct LogPolicy {
    kind: String,
    trigger: LogTrigger,
    roller: LogRoller,
}

struct LogTrigger {
    kind: String,
    limit: i32,
}
struct LogRoller {
    kind: String,
    pattern: String,
    base: i32,
    count: i32,
}

struct LogStdout {
    kind: String,
    encoder: LogEncoder,
}

struct LogEncoder {
    pattern: String,
}


fn out(s: &str, _f: Option<(u8, u8, u8)>) {
    #[cfg(not(target_os = "windows"))]
    e_utils::output!(rgb[_f, None] s);
    #[cfg(target_os = "windows")]
    println!("{}", s);
}