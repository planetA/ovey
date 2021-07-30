use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Virt<T> {
    pub(crate) real: T,
    pub(crate) virt: T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GidEntry {
    pub(crate) port: u16,
    pub(crate) idx: u32,
    pub(crate) subnet_prefix: u64,
    pub(crate) interface_id: u64,
}

impl GidEntry {
    pub(crate) fn new(idx: u32, subnet_prefix: u64, interface_id: u64) -> Self {
        GidEntry{
            port: 1,
            idx: idx,
            subnet_prefix,
            interface_id,
        }
    }

    pub(crate) fn is_same_addr(&self, other: &Self) -> bool {
        self.subnet_prefix == other.subnet_prefix &&
            self.interface_id == other.interface_id
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DeviceEntry {
    pub(crate) device: Uuid,
    pub(crate) guid: Option<Virt<u64>>,
    pub(crate) gid: Vec<Virt<GidEntry>>,
    pub(crate) lease: Instant,
}

impl DeviceEntry {
    pub(crate) fn new(device: Uuid) -> Self {
        Self{
            device: device,
            lease: Instant::now(),
            guid: None,
            gid: Vec::new(),
        }
    }

    pub(crate) fn is_gid_unique(&mut self, gid: Virt<GidEntry>) -> bool {
        self.gid.iter()
            .find(|e| e.virt.is_same_addr(&gid.virt) || e.real.is_same_addr(&gid.real))
            .is_none()
    }

    pub(crate) fn set_guid(&mut self, guid: Virt<u64>) -> &mut Self {
        self.guid = Some(guid);
        self
    }

    pub(crate) fn set_gid(&mut self, gid: Virt<GidEntry>) -> &mut Self {
        if !self.is_gid_unique(gid) {
            panic!("Duplicate gid");
        }

        self.gid.push(gid);
        self
    }
}

#[derive(Debug)]
pub(crate) struct DeviceTable(Vec<DeviceEntry>);

impl DeviceTable {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn by_device(&mut self, dev: Uuid) -> Option<&mut DeviceEntry> {
        self.0.iter_mut().find(|e| e.device == dev)
    }

    pub(crate) fn insert(&mut self, entry: DeviceEntry) {
        self.0.push(entry);
    }

    pub(crate) fn vec(&self) -> &Vec<DeviceEntry> {
        &self.0
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, DeviceEntry>
    {
        self.0.iter()
    }
}

pub(crate) struct NetworkState {
    pub(crate) devices: DeviceTable,
}

impl NetworkState {
    pub(crate) fn new() -> Self {
        NetworkState{
            devices: DeviceTable::new(),
        }
    }
}

pub(crate) struct CoordState {
    pub(crate) networks: Mutex<HashMap<Uuid, NetworkState>>,
}
