use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    process,
    sync::{Arc, Mutex},
    thread,
};

use crate::ClientType;
use crate::{check_and_mark_running, Args, Effect, LedData, LED_SIZE, UNIVERSE};
use crate::{effects::test::TestEffect, Color};
use sacn_unofficial::{packet::ACN_SDT_MULTICAST_PORT, source::SacnSource};
use zbus::{dbus_interface, ConnectionBuilder};

async fn create_dbus_connection(
    effect: &Arc<Mutex<Box<dyn Effect + Send>>>,
) -> Result<zbus::Connection, Box<dyn Error>> {
    let bus_interface = BusInterface {
        effect: Arc::clone(effect),
    };
    Ok(ConnectionBuilder::session()?
        .name("dev.rugmj.LedController")?
        .serve_at("/dev/rugmj/LedController", bus_interface)?
        .build()
        .await?)
}

pub async fn daemon(args: Args) {
    let file_lock = check_and_mark_running();

    if file_lock.is_err() {
        println!("There can only be one of us");
        panic!("Another instance is already running");
    }

    let effect = create_effect(&args);
    let effect = Arc::new(Mutex::new(effect));

    let _conn = create_dbus_connection(&effect).await.unwrap();

    let (mut src, dst_ip) = setup_sacn();
    loop {
        let (config, data) = {
            let mut effect = effect.lock().unwrap();
            (
                effect.get_config(),
                effect.update().unwrap_or(Some([Color::BLACK; LED_SIZE])),
            )
        };

        if let Some(data) = data {
            if let Err(err) = send_data(&mut src, dst_ip, &data) {
                println!("Error: {:?}", err);
            }
        }

        thread::sleep(config.delay);
    }
}

fn create_effect(args: &Args) -> Box<dyn Effect + Send> {
    match &args.effect {
        Some(client_type) => (*client_type).into_effect(),
        None => {
            if args.test {
                Box::new(TestEffect::new())
            } else {
                ClientType::RainbowEffect.into_effect()
            }
        }
    }
}

#[derive(Debug)]
enum SendDataErr {
    IncorrectLength,
    SacnError,
}

fn send_data(src: &mut SacnSource, dst_ip: SocketAddr, data: &LedData) -> Result<(), SendDataErr> {
    if data.len() != LED_SIZE {
        return Err(SendDataErr::IncorrectLength);
    }
    let data = data
        .iter()
        .flat_map(|&c| vec![c.2, c.0, c.1])
        .collect::<Vec<u8>>();
    let data_slice = data.as_slice();

    if src
        .send(&[UNIVERSE], data_slice, None, Some(dst_ip), None)
        .is_err()
    {
        return Err(SendDataErr::SacnError);
    }

    Ok(())
}

fn setup_sacn() -> (SacnSource, SocketAddr) {
    let destination_address = SocketAddr::new(
        IpAddr::V4("192.168.1.73".parse().unwrap()),
        ACN_SDT_MULTICAST_PORT,
    );
    let dst_ip = destination_address;
    let local_addr = SocketAddr::new(
        IpAddr::V4("0.0.0.0".parse().unwrap()),
        ACN_SDT_MULTICAST_PORT + 1,
    );

    let mut src = SacnSource::with_ip("Source", local_addr).unwrap();

    src.register_universe(UNIVERSE).unwrap();

    (src, dst_ip)
}

struct BusInterface {
    effect: Arc<Mutex<Box<dyn Effect + Send>>>,
}

#[dbus_interface(name = "dev.rugmj.LedController1")]
impl BusInterface {
    fn set_effect(&mut self, new_client_type: ClientType) {
        let mut effect = self.effect.lock().unwrap();
        *effect = new_client_type.into_effect();
    }

    fn end_daemon(&self) {
        process::exit(0)
    }
}
