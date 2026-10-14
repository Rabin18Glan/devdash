use sysinfo::Disks;

#[derive(Clone, Debug, PartialEq)]
pub struct StorageItem {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_gb: f64,
    pub used_gb: f64,
    pub free_gb: f64,
    pub usage_percent: f32,
    pub is_removable: bool,
}

pub fn scan_storage() -> Vec<StorageItem> {
    let disks = Disks::new_with_refreshed_list();
    let mut items = Vec::new();

    for disk in &disks {
        let total_bytes = disk.total_space();
        let free_bytes = disk.available_space();
        let used_bytes = total_bytes.saturating_sub(free_bytes);

        let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let free_gb = free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let used_gb = used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        let usage_percent = if total_bytes > 0 {
            ((used_bytes as f64 / total_bytes as f64) * 100.0) as f32
        } else {
            0.0
        };

        let name = disk.name().to_string_lossy().to_string();
        let mount_point = disk.mount_point().to_string_lossy().to_string();
        let file_system = disk.file_system().to_string_lossy().to_string();

        items.push(StorageItem {
            name,
            mount_point,
            file_system,
            total_gb,
            used_gb,
            free_gb,
            usage_percent,
            is_removable: disk.is_removable(),
        });
    }

    // Default sort by Total GB descending (primary disks first)
    items.sort_by(|a, b| {
        b.total_gb
            .partial_cmp(&a.total_gb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    items
}
