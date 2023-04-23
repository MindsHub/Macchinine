use std::{
    collections::{HashMap, HashSet},
    pin::Pin, sync::mpsc::Sender,
};

use bluer::{gatt::remote::Characteristic, AdapterEvent, Address, Device, Error, Uuid};
use colored::Colorize;
use futures::{pin_mut, Future, StreamExt};

use crate::egui::RobotEvent;

//use crate::{find_our_characteristic, exercise_characteristic, SERVICE_UUID, CHARACTERISTIC_UUID};
/*pub trait BlFunc{
    fn exec(&self, char: Characteristic);
}
impl<F, T> BlFunc for F
where
    F: Fn(Characteristic)->T,
    T: Future<Output = ()>{
    fn exec(&self, char: Characteristic)
    {
        let y = Handle::current();

        y.spawn(future)
        y.block_on(self(char));

    }
}*/
struct BleDevice {
    service: Uuid,
    characteristic: Uuid,
    //address: Address,
    run: Box<dyn Fn(Characteristic, Sender<RobotEvent>) -> Pin<Box< dyn Future<Output = bluer::Result<()>> >>> ,
}

pub struct Bluetooth{
    accepted: HashMap<Address, BleDevice>,
    sender: Sender<RobotEvent>,
}

impl Bluetooth {
    pub fn add_device<Fut>(
        & mut self,
        service: Uuid,
        characteristic: Uuid,
        address: Address,
        f: impl Fn(Characteristic, Sender<RobotEvent>) -> Fut + 'static,
    ) where
        Fut: Future<Output = bluer::Result<()>> + 'static,
    {
        let device = BleDevice {
            characteristic,
            service,
            run: Box::new(move |x, s| Box::pin(f(x, s))),
        };
        self.accepted.insert(address, device);
    }
    pub fn new(sender: Sender<RobotEvent>) -> Self {
        Bluetooth {
            accepted: HashMap::new(),
            sender,
        }
    }

    async fn connect_device(device: &Device) -> Result<(), Error> {
        println!("    Connecting...");
        let mut retries = 2;
        loop {
            match device.connect().await {
                Ok(()) => break,
                Err(err) if retries > 0 => {
                    println!("    Connect error: {}", &err);
                    retries -= 1;
                }
                Err(err) => return Err(err),
            }
        }
        println!("    Connected");
        Ok(())
    }

    pub async fn filter_and_run(&self, device: &Device) -> Result<Option<Characteristic>, Error> {
        let addr = device.address();
        println!("Discovered device {addr}");
        println!("Name {:?}", device.name().await?);
        //check if we have it in our available connections
        let ble_device = self.accepted.get(&addr);
        if ble_device.is_none() {
            return Ok(None);
        }
        let ble_device = ble_device.unwrap();

        //connetti
        if !device.is_connected().await? {
            Self::connect_device(device).await?;
        } else {
            println!("    Already connected");
        }

        println!("getting services");
        println!("{:?}", device.services().await?);
        for service in device.services().await? {
            if service.uuid().await? == ble_device.service {
                println!("    Found our service!");
                for char in service.characteristics().await? {
                    let uuid = char.uuid().await?;
                    println!(
                        "    Characteristic UUID: {}-{}",
                        &uuid, ble_device.characteristic
                    );

                    if uuid == ble_device.characteristic {
                        println!("    Found our characteristic!");
                        (ble_device.run)(char, self.sender.clone()).await?;

                        return Ok(None);
                    }
                }
            }
        }
        Ok(None)
    }

    pub async fn scan(&self) -> Result<(), Error> {
        //get bl adapter
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
//        adapter.set_powered(false).await?;
        //turn on adapter
//        adapter.set_powered(true).await?;
        println!(
            "Discovering on Bluetooth adapter {} with address {}\n",
            adapter.name(),
            adapter.address().await?
        );

        // start scan
        let discover = adapter.discover_devices().await?;
        // pin discover (not going out of scope in async)
        pin_mut!(discover);
        let mut discovered = HashSet::new();
        while let Some(evt) = discover.next().await {
            match evt {
                AdapterEvent::DeviceAdded(addr) => {
                    discovered.insert(addr);
                    println!("discovered={}", discovered.len().to_string().blue());
                    // if another device connected, let's try to find our characteristics
                    let device = adapter.device(addr)?;
                    self.filter_and_run(&device).await?;
                }
                //AdapterEvent::PropertyChanged()
                AdapterEvent::DeviceRemoved(addr) => {
                    //discovered-=1;
                    println!("Removed device {addr}");
                }
                AdapterEvent::PropertyChanged(_) => {}
            }
        }

        Ok(())
    }
}
