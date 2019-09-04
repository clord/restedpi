use crate::i2c_bus::Result;
use rppal::i2c::I2c;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::vec::Vec;

type Address = u16;

pub struct I2cMessage {
    pub action: I2cAction,
    pub address: Address,
    pub command: u8,
}

pub enum I2cAction {
    Write {
        parameters: Vec<u8>,
        response: Sender<Result<()>>,
    },
    Read {
        size: usize,
        response: Sender<Result<Vec<u8>>>,
    },
}

fn next_message(
    current_address: &mut Option<Address>,
    i2c: &mut I2c,
    receiver: &Receiver<I2cMessage>,
) {
    let next = receiver.recv().unwrap();
    if *current_address != Some(next.address) {
        *current_address = Some(next.address);
        i2c.set_slave_address(next.address)
            .expect("failed to set slave address");
    }
    match next.action {
        I2cAction::Write {
            response,
            parameters,
        } => {
            let result = i2c.block_write(next.command, &parameters);
            response.send(Ok(())).expect("failed to send");
        }
        I2cAction::Read { size, response } => {
            let mut vec = Vec::new();
            vec.resize(size, 0u8);
            match i2c.block_read(next.command, &mut vec) {
                Ok(()) => {
                    response.send(Ok(vec)).expect("failed to send");
                }
                Err(e) => {
                    response
                        .send(Err(crate::i2c_bus::error::Error::I2cError(e)))
                        .expect("failed to send error");
                }
            };
        }
    };
}

pub fn start() -> Sender<I2cMessage> {
    let mut current_address: Option<Address> = None;
    let (sender, receiver) = channel::<I2cMessage>();

    thread::spawn(move || match I2c::new() {
        Ok(mut i2c) => loop {
            next_message(&mut current_address, &mut i2c, &receiver);
        },
        Err(err) => {
            println!("ERROR: The I2C bus connected to pins 3 and 5 is disabled by default.");
            println!("       You can enable it through `sudo raspi-config`, or by manually adding `dtparam=i2c_arm=on` to `/boot/config.txt`. ");
            println!("       Remember to reboot the Raspberry Pi afterwards.");
            panic!("Aborting")
        }
    });
    sender
}
