use std::ffi::CString;

use crate::topology::bindgen::*;
use crate::topology::helper;
use crate::{DeviceError, DeviceResult};

pub trait Hwloc {
    fn init_topology(&mut self) -> DeviceResult<()>;
    fn set_io_types_filter(&mut self, filter: hwloc_type_filter_e) -> DeviceResult<()>;
    fn load_topology(&mut self) -> DeviceResult<()>;
    fn set_topology_from_xml(&mut self, xml_path: &str) -> DeviceResult<()>;
    fn get_common_ancestor_obj(&self, dev1bdf: &str, dev2bdf: &str) -> DeviceResult<hwloc_obj_t>;
    fn destroy_topology(&mut self);
}

pub struct HwlocTopology {
    topology: hwloc_topology_t,
}

impl HwlocTopology {
    pub fn new() -> Self {
        Self {
            topology: std::ptr::null_mut(),
        }
    }
}

impl Hwloc for HwlocTopology {
    fn init_topology(&mut self) -> DeviceResult<()> {
        unsafe {
            if hwloc_topology_init(&mut self.topology) == 0 {
                Ok(())
            } else {
                Err(DeviceError::hwloc_error(
                    "couldn't initialize hwloc library",
                ))
            }
        }
    }

    fn set_io_types_filter(&mut self, filter: hwloc_type_filter_e) -> DeviceResult<()> {
        unsafe {
            if hwloc_topology_set_io_types_filter(self.topology, filter) == 0 {
                Ok(())
            } else {
                Err(DeviceError::hwloc_error("couldn't set filter"))
            }
        }
    }

    fn load_topology(&mut self) -> DeviceResult<()> {
        unsafe {
            if hwloc_topology_load(self.topology) == 0 {
                Ok(())
            } else {
                Err(DeviceError::hwloc_error("couldn't load topology"))
            }
        }
    }

    fn set_topology_from_xml(&mut self, xmlpath: &str) -> DeviceResult<()> {
        unsafe {
            let xml_path_cstr = CString::new(xmlpath).unwrap();
            if hwloc_topology_set_xml(self.topology, xml_path_cstr.as_ptr()) == 0 {
                Ok(())
            } else {
                Err(DeviceError::hwloc_error("couldn't set topology from xml"))
            }
        }
    }

    fn get_common_ancestor_obj(&self, dev1bdf: &str, dev2bdf: &str) -> DeviceResult<hwloc_obj_t> {
        unsafe {
            let dev1_obj = helper::hwloc_get_pcidev_by_busidstring(self.topology, dev1bdf);
            if dev1_obj.is_null() {
                return Err(DeviceError::hwloc_error(format!(
                    "couldn't find object with the bus id {dev1bdf}"
                )));
            }

            let dev2_obj = helper::hwloc_get_pcidev_by_busidstring(self.topology, dev2bdf);
            if dev2_obj.is_null() {
                return Err(DeviceError::hwloc_error(format!(
                    "couldn't find object with the bus id {dev2bdf}"
                )));
            }

            let ancestor = helper::hwloc_get_common_ancestor_obj(dev1_obj, dev2_obj);
            if ancestor.is_null() {
                return Err(DeviceError::hwloc_error(format!(
                    "couldn't find a common ancestor for objects {dev1bdf} and {dev2bdf}"
                )));
            }

            Ok(ancestor)
        }
    }

    fn destroy_topology(&mut self) {
        unsafe {
            if !self.topology.is_null() {
                hwloc_topology_destroy(self.topology);
                self.topology = std::ptr::null_mut();
            }
        }
    }
}
