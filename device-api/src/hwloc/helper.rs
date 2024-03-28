#![allow(warnings)]

use std::os::raw::c_int;
use std::ptr;

use lazy_static::lazy_static;
use regex::Regex;

use crate::hwloc::bindgen::*;

pub unsafe fn hwloc_get_common_ancestor_obj(
    mut obj1: hwloc_obj_t,
    mut obj2: hwloc_obj_t,
) -> hwloc_obj_t {
    while obj1 != obj2 {
        while (*obj1).depth > (*obj2).depth {
            obj1 = (*obj1).parent;
        }
        while (*obj2).depth > (*obj1).depth {
            obj2 = (*obj2).parent;
        }
        if obj1 != obj2 && (*obj1).depth == (*obj2).depth {
            obj1 = (*obj1).parent;
            obj2 = (*obj2).parent;
        }
    }
    obj1
}

/*
static __hwloc_inline hwloc_obj_t
hwloc_get_pcidev_by_busid(hwloc_topology_t topology,
              unsigned domain, unsigned bus, unsigned dev, unsigned func)
{
  hwloc_obj_t obj = NULL;
  while ((obj = hwloc_get_next_pcidev(topology, obj)) != NULL) {
    if (obj->attr->pcidev.domain == domain
    && obj->attr->pcidev.bus == bus
    && obj->attr->pcidev.dev == dev
    && obj->attr->pcidev.func == func)
      return obj;
  }
  return NULL;
}
*/

unsafe fn hwloc_get_pcidev_by_busid(
    topology: hwloc_topology_t,
    domain: u16,
    bus: u8,
    dev: u8,
    func: u8,
) -> hwloc_obj_t {
    let mut obj = hwloc_get_next_pcidev(topology, ptr::null_mut());
    while obj != ptr::null_mut() {
        if (*(*obj).attr).pcidev.domain == domain
            && (*(*obj).attr).pcidev.bus == bus
            && (*(*obj).attr).pcidev.dev == dev
            && (*(*obj).attr).pcidev.func == func
        {
            return obj;
        }
        obj = hwloc_get_next_pcidev(topology, obj)
    }

    ptr::null_mut()
}

/*
hwloc_obj_t
hwloc_get_pcidev_by_busidstring(hwloc_topology_t topology, const char *busid)
{
  unsigned domain = 0; /* default */
  unsigned bus, dev, func;

  if (sscanf(busid, "%x:%x.%x", &bus, &dev, &func) != 3
      && sscanf(busid, "%x:%x:%x.%x", &domain, &bus, &dev, &func) != 4) {
    errno = EINVAL;
    return NULL;
  }

  return hwloc_get_pcidev_by_busid(topology, domain, bus, dev, func);
}


*/

lazy_static! {
    static ref BDF_NOTATION_PATTERN: Regex = Regex::new(r"^(?:(?P<domain>[0-9a-fA-F]+):)?(?P<bus>[0-9a-fA-F]+):(?P<dev>[0-9a-fA-F]+)\.(?P<func>[0-9a-fA-F]+)").unwrap();
}

pub unsafe fn hwloc_get_pcidev_by_busidstring(
    topology: hwloc_topology_t,
    busid: &str,
) -> hwloc_obj_t {
    return match BDF_NOTATION_PATTERN.captures(busid) {
        Some(caps) => {
            let domain = caps
                .name("domain")
                .map_or(0, |m| u16::from_str_radix(m.as_str(), 16).unwrap_or(0));
            let bus = u8::from_str_radix(caps.name("bus").unwrap().as_str(), 16).unwrap();
            let dev = u8::from_str_radix(caps.name("dev").unwrap().as_str(), 16).unwrap();
            let func = u8::from_str_radix(caps.name("func").unwrap().as_str(), 16).unwrap();
            hwloc_get_pcidev_by_busid(topology, domain, bus, dev, func)
        }
        None => ptr::null_mut(),
    };
}

/*
static __hwloc_inline hwloc_obj_t
hwloc_get_next_obj_by_depth (hwloc_topology_t topology, int depth, hwloc_obj_t prev)
{
  if (!prev)
    return hwloc_get_obj_by_depth (topology, depth, 0);
  if (prev->depth != depth)
    return NULL;
  return prev->next_cousin;
}
*/
unsafe fn hwloc_get_next_obj_by_depth(
    topology: hwloc_topology_t,
    depth: c_int,
    prev: hwloc_obj_t,
) -> hwloc_obj_t {
    if prev.is_null() {
        return hwloc_get_obj_by_depth(topology, depth, 0);
    }

    if (*prev).depth != depth {
        return ptr::null_mut();
    }

    (*prev).next_cousin
}
/*
static __hwloc_inline hwloc_obj_t
hwloc_get_next_obj_by_type (hwloc_topology_t topology, hwloc_obj_type_t type,
                hwloc_obj_t prev)
{
  int depth = hwloc_get_type_depth(topology, type);
  if (depth == HWLOC_TYPE_DEPTH_UNKNOWN || depth == HWLOC_TYPE_DEPTH_MULTIPLE)
    return NULL;
  return hwloc_get_next_obj_by_depth (topology, depth, prev);
}
*/
unsafe fn hwloc_get_next_obj_by_type(
    topology: hwloc_topology_t,
    obj_type: hwloc_obj_type_t,
    prev: hwloc_obj_t,
) -> hwloc_obj_t {
    let depth = hwloc_get_type_depth(topology, obj_type);
    return match depth {
        hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_UNKNOWN => ptr::null_mut(),
        hwloc_get_type_depth_e_HWLOC_TYPE_DEPTH_MULTIPLE => ptr::null_mut(),
        d => hwloc_get_next_obj_by_depth(topology, d, prev),
    };
}

/*
static __hwloc_inline hwloc_obj_t
hwloc_get_next_pcidev(hwloc_topology_t topology, hwloc_obj_t prev)
{
  return hwloc_get_next_obj_by_type(topology, HWLOC_OBJ_PCI_DEVICE, prev);
}
 */

unsafe fn hwloc_get_next_pcidev(topology: hwloc_topology_t, prev: hwloc_obj_t) -> hwloc_obj_t {
    hwloc_get_next_obj_by_type(topology, hwloc_obj_type_t_HWLOC_OBJ_PCI_DEVICE, prev)
}
