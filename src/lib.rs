use std::{time::Duration, io::{Read, Write}};
use napi_derive::napi;
use napi::{ Result, Error, bindgen_prelude::ObjectFinalize};
use serde::{Deserialize, Serialize};
use serialport::TTYPort;
use serde_json::{Deserializer, Serializer};

#[napi(custom_finalize)]
pub struct Pyboard {
    port: TTYPort
}

#[derive(Serialize, Deserialize)]
struct Command {
    command: String,
    path: String
}

#[derive(Deserialize)]
enum PythonResult<T> {
    #[serde(rename = "success")]
    Success(T),
    #[serde(rename = "error")]
    Error(String)
}

#[napi]
impl Pyboard {
    #[napi(constructor)]
    pub fn new(path: String, baud_rate: u32) -> Result<Pyboard> {
        Ok(Pyboard {
            port: serialport::new(path, baud_rate)
                .timeout(Duration::from_secs(5))
                .open_native()
                .map_err(|err| Error::from_reason(err.to_string()))
                .and_then(|port| setup(port))?
        })
    }

    #[napi]
    pub fn cat(&mut self, path: String) -> Result<String> {
        self.send(Command {
            command: "cat".to_string(),
            path
        })?;
        self.recv::<PythonResult<String>>()?.into()
    }

    #[napi]
    pub fn ls(&mut self, path: String) -> Result<Vec<String>> {
        self.send(Command {
            command: "ls".to_string(),
            path
        })?;
        self.recv::<PythonResult<Vec<String>>>()?.into()
    }

    #[napi]
    pub fn exists(&mut self, path: String) -> Result<bool> {
        self.send(Command {
            command: "exists".to_string(),
            path
        })?;
        self.recv::<PythonResult<bool>>()?.into()
    }

    fn recv<'a, T: Deserialize<'a>>(&mut self) -> Result<T> {
        let mut deserializer = Deserializer::from_reader(&mut self.port);
        T::deserialize(&mut deserializer)
            .map_err(|err| Error::from_reason(err.to_string()))
    }

    fn send<'a, T: Serialize + Deserialize<'a>>(&mut self, value: T) -> Result<()> {
        let mut serializer = Serializer::new(&mut self.port);
        value.serialize(&mut serializer)
            .map_err(|err| Error::from_reason(err.to_string()))?;
        self.port.write_all(&[b'\n'])
            .map_err(|err| Error::from_reason(err.to_string()))?;
        self.recv::<T>()
            .map(|_| ())
    }
}

fn setup(mut port: TTYPort) -> Result<TTYPort> {

    port.write_all(&[0b00000011, 0b00000011, 0b00000101]).and_then(|_| {
        port.write_all(include_bytes!("main.py"))
    }).and_then(|_| {
        port.write_all(&[0b00000100])
    })
    .map_err(|err| Error::from_reason(err.to_string()))?;

    let mut buffer = [0; 2];
    let mut prefix = true;

    let mut error = loop {
        buffer.rotate_right(1);
        port.read_exact(&mut buffer[..1]).unwrap();
        if buffer[1] == b'\n' {
            if buffer[0] == b'=' {
                prefix = false;
            } else if !prefix {
                if buffer[0] == b'#' {
                    return Ok(port)
                } else {
                    break String::from(buffer[0] as char)
                }
            }
        }
    };

    while port.read_exact(&mut buffer[..1]).is_ok() {
        error.push(buffer[0] as char);
    }

    Err(Error::from_reason(error))
}

impl<T> Into<Result<T>> for PythonResult<T> {
    fn into(self) -> Result<T> {
        match self {
            PythonResult::Success(value) => Ok(value),
            PythonResult::Error(error) => Err(Error::from_reason(error.to_string())),
        }
    }
}

impl ObjectFinalize for Pyboard {
    fn finalize(mut self, _: napi::Env) -> Result<()> {
        self.send(Command {
            command: "exit".to_string(),
            path: "".to_string(),
        }).ok();
        Ok(())
    }
}