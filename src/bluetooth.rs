use std::{collections::{HashMap, HashSet}, pin::Pin};

use bluer::{Uuid, Address, Error, AdapterEvent, Device, gatt::remote::Characteristic};
use colored::Colorize;
use futures::{pin_mut, StreamExt, Future, executor::block_on};
use tokio::runtime::Handle;

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
struct BleDevice{
    characteristic: Uuid,
    address: Address,
    run: Box<dyn Fn(Characteristic)->Pin<Box<dyn Future<Output = ()>>>>,
}


pub struct Bluetooth{

    accepted: HashMap<Uuid, BleDevice>
}

impl Bluetooth{
    pub fn add_device<Fut>(&mut self, service: Uuid, characteristic: Uuid, address: Address, f: impl Fn(Characteristic)->Fut + 'static)
        where Fut: Future<Output = ()> + 'static, 
    {
        let device= BleDevice{characteristic, address, run: Box::new(move |x| { Box::pin(f(x)) })};
        self.accepted.insert(service, device);
    }
    pub fn new()->Self{
        Bluetooth{accepted: HashMap::new()}
    }

    
    async fn connect_device(device: &Device)-> Result<(), Error>{
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
        let uuids = device.uuids().await?.unwrap_or_default();

        println!("Discovered device {} with service UUIDs: {:?}", addr, uuids);
        println!("Name {:?}", device.name().await?);
        let md = device.manufacturer_data().await?;
        println!("    Manufacturer data: {:x?}", &md);
        for uuid in uuids{
            // if not provided
            let ble_device = self.accepted.get(&uuid);
            
            if ble_device.is_none() {
                continue;
            }
            let ble_device = ble_device.unwrap();
            if addr!=ble_device.address{
                println!("wrong address");
                continue;
            }

            println!("Device provides one of our services!");

            //try to connect
            if !device.is_connected().await? {
                Self::connect_device(device).await?;
            } else {
                println!("    Already connected");
            }
            println!("getting services");
            println!("{:?} {:?}", device.services().await?, device.uuids().await);
            for service in device.services().await? {
                let uuid = service.uuid().await?;
                println!("    Service UUID: {}", &uuid);
                let ble_device= self.accepted.get(&uuid);
                if let Some(ble_device)  = ble_device{
                    println!("    Found our service!");
                    for char in service.characteristics().await? {
                        let uuid = char.uuid().await?;
                        println!("    Characteristic UUID: {}", &uuid);
                        if uuid == ble_device.characteristic {
                            println!("    Found our characteristic!");
                            (ble_device.run)(char).await;

                            
                            return Ok(None);
                        }
                    }
                }
            }
            
        }
        Ok(None)
    }

    

    pub async fn scan (&self) -> Result<(), Error>{
        //get bl adapter
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;

        //turn on adapter
        adapter.set_powered(true).await?;
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
                AdapterEvent::PropertyChanged(_)=>{}
            }
        }
        
        Ok(())

    }
}