use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use std::convert::TryInto;
use std::convert::TryFrom;

use uuid::Uuid;

use liboveyutil::types::Gid;
use crate::rest::errors::CoordinatorRestError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct Virt<T> {
    pub(crate) real: T,
    pub(crate) virt: T,
}

impl<T> Virt<T> {
    pub(crate) fn new(real: T, virt: T) -> Self {
        Virt{
            real: real,
            virt: virt,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GidEntry {
    pub(crate) idx: u32,
    pub(crate) gid: Gid,
}

impl GidEntry {
    pub(crate) fn new(idx: u32, gid: Gid) -> Self {
        GidEntry{
            idx: idx,
            gid: gid,
        }
    }

    pub(crate) fn is_same_addr(&self, other: &Self) -> bool {
        self.gid.subnet_prefix == other.gid.subnet_prefix &&
            self.gid.interface_id == other.gid.interface_id
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PortEntry {
    pub(crate) id: Virt<u16>,
    pub(crate) lid: Option<Virt<u32>>,
	  pub(crate) pkey_tbl_len: u32,
	  pub(crate) gid_tbl_len: u32,
	  pub(crate) core_cap_flags: u32,
	  pub(crate) max_mad_size: u32,
    gid: Vec<Virt<GidEntry>>,
}

impl PortEntry {
    pub(crate) fn new(id: Virt<u16>) -> Self {
        PortEntry{
            id: id,
            lid: None,
            gid: Vec::new(),
            ..Default::default()
        }
    }

    pub(crate) fn set_pkey_tbl_len(&mut self, pkey_tbl_len: u32) -> &mut Self {
        self.pkey_tbl_len = pkey_tbl_len;
        self
    }

    pub(crate) fn set_gid_tbl_len(&mut self, gid_tbl_len: u32) -> &mut Self {
        self.gid_tbl_len = gid_tbl_len;
        self
    }

    pub(crate) fn set_core_cap_flags(&mut self, core_cap_flags: u32) -> &mut Self {
        self.core_cap_flags = core_cap_flags;
        self
    }

    pub(crate) fn set_max_mad_size(&mut self, max_mad_size: u32) -> &mut Self {
        self.max_mad_size = max_mad_size;
        self
    }

    pub(crate) fn add_gid(&mut self, gid: Virt<GidEntry>) -> Result<(), CoordinatorRestError> {
        if !self.is_gid_unique(gid) {
            return Err(CoordinatorRestError::GidConflict);
        }

        if self.gid.len() >= self.gid_tbl_len.try_into().unwrap() {
            return Err(CoordinatorRestError::GidConflict);
        }

        self.gid.push(gid);
        Ok(())
    }

    pub(crate) fn is_gid_unique(&self, gid: Virt<GidEntry>) -> bool {
        self.gid.iter()
            .find(|e| e.virt.is_same_addr(&gid.virt) || e.real.is_same_addr(&gid.real))
            .is_none()
    }

    pub(crate) fn iter_gid(&self) -> std::slice::Iter<'_, Virt<GidEntry>>
    {
        self.gid.iter()
    }

    pub(crate) fn iter_gid_mut(&mut self) -> std::slice::IterMut<'_, Virt<GidEntry>>
    {
        self.gid.iter_mut()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct QpEntry {
    pub(crate) qpn: Virt<u32>,
}

#[derive(Clone, Debug)]
pub(crate) struct DeviceEntry {
    pub(crate) device: Uuid,
    pub(crate) guid: Option<Virt<u64>>,
    ports: Vec<PortEntry>,
    qps: Vec<QpEntry>,
    pub(crate) lease: Instant,
}

impl DeviceEntry {
    pub(crate) fn new(device: Uuid) -> Self {
        Self{
            device: device,
            lease: Instant::now(),
            guid: None,
            ports: Vec::new(),
            qps: Vec::new(),
        }
    }

    pub(crate) fn is_gid_unique(&self, gid: Virt<GidEntry>) -> bool {
        self.ports.iter()
            .find(|p| !p.is_gid_unique(gid))
            .is_none()
    }

    pub(crate) fn set_guid(&mut self, guid: Virt<u64>) -> &mut Self {
        self.guid = Some(guid);
        self
    }

    /// Return mutable reference to a port.
    pub(crate) fn get_port_mut(&mut self, port_id: u16) -> Option<&mut PortEntry> {
        // Remember, the port indicies start from 1
        self.ports.get_mut((port_id - 1) as usize)
    }

    pub(crate) fn add_port(&mut self, real_port: u16) -> &mut PortEntry {
        // Find the next available index
        // We count port IDs from 1
        let virt_id = u16::try_from(self.ports.len() + 1).unwrap();
        let port = PortEntry::new(Virt::new(real_port, virt_id));
        self.ports.push(port);
        self.ports.last_mut().unwrap()
    }

    pub(crate) fn iter_port(&self) -> std::slice::Iter<'_, PortEntry>
    {
        self.ports.iter()
    }

    pub(crate) fn add_qp(&mut self, qpn: Virt<u32>) -> &mut QpEntry {
        self.qps.push(QpEntry{
            qpn: qpn,
        });
        self.qps.last_mut().unwrap()
    }

    pub(crate) fn iter_qps(&self) -> std::slice::Iter<'_, QpEntry>
    {
        self.qps.iter()
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

    pub(crate) fn is_gid_unique(&self, gid: Virt<GidEntry>) -> bool {
        self.devices.iter()
            .find(|device| !device.is_gid_unique(gid))
            .is_none()
    }
}

pub(crate) struct CoordState {
    networks: Mutex<HashMap<Uuid, NetworkState>>,
}

impl CoordState {
    pub(crate) fn new() -> Self {
        CoordState{
            networks: Mutex::new(HashMap::new())
        }
    }

    pub(crate) fn with_network<R, F>(&self, network_uuid: Uuid, mut f: F) -> Result<R, CoordinatorRestError>
    where
        F: FnMut(&mut NetworkState) -> Result<R, CoordinatorRestError>
    {
        let mut networks = self.networks.lock().unwrap();
        let network: &mut NetworkState = networks
            .get_mut(&network_uuid)
            .ok_or(CoordinatorRestError::NetworkUuidNotFound(network_uuid))?;

        f(network)
    }

    pub(crate) fn with_network_insert<R, F>(&self, network_uuid: Uuid, mut f: F)
                                            -> Result<R, CoordinatorRestError>
    where
        F: FnMut(&mut NetworkState) -> Result<R, CoordinatorRestError>
    {
        let mut networks = self.networks.lock().unwrap();
        let network = networks.entry(network_uuid).or_insert(NetworkState::new());

        f(network)
    }
}
