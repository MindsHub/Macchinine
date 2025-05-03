use std::time::Duration;

use btleplug::{
    api::{
        BDAddr, Central, CentralEvent, Characteristic, Manager as _, Peripheral as _, ScanFilter    },
    platform::{Manager, Peripheral},
};
use futures::stream::StreamExt;
use uuid::Uuid;

use crate::setup::Error;


pub async fn connect(
    address: BDAddr,
    service: Uuid,
    characteristic: Uuid,
) -> Result<(Peripheral, Characteristic), Error> {
    log::info!("Search device and connect");
    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    let mut events = central.events().await?;
    // start scanning for devices, while filtering for the services we want
    let filter = ScanFilter {
        services: vec![service],
    };

    central.start_scan(filter).await?;
    let ret = tokio::time::timeout(Duration::from_secs(20), async{
        while let Some(e) = events.next().await {
            log::error!("Event: {:?}", e);
            match e {
                CentralEvent::DeviceDiscovered(id) => {
                    let peripheral = central.peripheral(&id).await.unwrap();
                    if peripheral.address() != address {
                        continue;
                    }
                    peripheral.connect().await?;
                    peripheral.discover_services().await?;
                    if peripheral.services().iter().find(|x| x.uuid==service).is_none() {
                        peripheral.disconnect().await?;
                        log::error!("Characteristic not found");
                        continue;
                    }
                    if let Some(x) = peripheral.characteristics().iter().find(|x| x.uuid==characteristic){
                        
                        log::info!("Connected to device: {:?}", peripheral.address());
                        
                        return  Ok((peripheral, x.clone()));
                    } else{
                        peripheral.disconnect().await?;
                        log::error!("Characteristic not found");
                        continue;
                    }
                }
                _ => {}
            }
        }
        Err(Error::NotFound)
    }).await.unwrap_or_else(|_| {
        log::error!("Timeout");
        Err(Error::NotFound)
    });
    central.stop_scan().await?;
    ret
    
}