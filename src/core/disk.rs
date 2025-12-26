use sysinfo::Disks;

#[derive(Clone, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
}

pub fn get_disks() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    
    disks.iter()
        .map(|disk| DiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
        })
        .collect()
}
