use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    process,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::effects::test::TestEffect;
use crate::ClientType;
use crate::{check_and_mark_running, Args, Effect, LedData, LED_SIZE, UNIVERSE};
use sacn_unofficial::{packet::ACN_SDT_MULTICAST_PORT, source::SacnSource};
use zbus::{dbus_interface, ConnectionBuilder};

pub(crate) async fn daemon(args: Args) -> Result<(), Box<dyn Error>> {
    let file_lock = check_and_mark_running();

    if file_lock.is_err() {
        println!("There can only be one of us");
        panic!("Another instance is already running");
    }

    let effect = create_effect(&args);
    let effect = Arc::new(Mutex::new(effect));

    let bus_interface = BusInterface {
        effect: Arc::clone(&effect),
    };

    let _conn = ConnectionBuilder::session()?
        .name("dev.rugmj.LedController")?
        .serve_at("/dev/rugmj/LedController", bus_interface)?
        .build()
        .await?;

    let (mut src, dst_ip) = setup_sacn();
    loop {
        let data = {
            let mut effect = effect.lock().unwrap();
            effect.update().unwrap_or([(0, 0, 0); LED_SIZE].to_vec())
        };

        if let Err(err) = send_data(&mut src, dst_ip, &data) {
            println!("Error: {:?}", err);
        }
        thread::sleep(Duration::from_millis(10))
    }
}

fn create_effect(args: &Args) -> Box<dyn Effect + Send + Sync> {
    match &args.effect {
        Some(client_type) => client_type.clone().into_effect(),
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
        .flat_map(|&(r, g, b)| vec![b, r, g])
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
    effect: Arc<Mutex<Box<dyn Effect + Send + Sync>>>,
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
