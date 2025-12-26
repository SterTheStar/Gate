use std::path::PathBuf;
use std::thread;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Default)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

#[derive(Clone, Debug)]
pub struct ScanProgress {
    pub files_scanned: u64,
    pub bytes_scanned: u64,
    pub current_path: String,
    pub start_time: std::time::Instant,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            files_scanned: 0,
            bytes_scanned: 0,
            current_path: String::new(),
            start_time: std::time::Instant::now(),
        }
    }
}

pub enum ScanMessage {
    Progress(ScanProgress),
    Completed(FileNode),
    Error(String),
}

pub struct Scanner {
    rx: Receiver<ScanMessage>,
}

impl Scanner {
    pub fn new(path: PathBuf) -> Self {
        let (tx, rx) = unbounded();
        thread::spawn(move || {
            let progress = Arc::new(Mutex::new(ScanProgress::default()));
            let tx_clone = tx.clone();
            let progress_clone = progress.clone();
            
            match std::panic::catch_unwind(move || {
                scan_recursive(&path, &tx_clone, &progress_clone)
            }) {
                Ok(root) => {
                    let _ = tx.send(ScanMessage::Completed(root));
                }
                Err(_) => {
                    let _ = tx.send(ScanMessage::Error("Scan panicked".to_string()));
                }
            }
        });
        Self { rx }
    }

    pub fn try_recv(&self) -> Option<ScanMessage> {
        self.rx.try_recv().ok()
    }
}

fn scan_recursive(
    path: &PathBuf, 
    tx: &Sender<ScanMessage>, 
    progress: &Arc<Mutex<ScanProgress>>
) -> FileNode {
    let mut node = FileNode {
        name: path.file_name().unwrap_or(path.as_os_str()).to_string_lossy().to_string(),
        path: path.clone(),
        size: 0,
        is_dir: path.is_dir(),
        children: vec![],
    };

    {
        if let Ok(mut p) = progress.lock() {
            p.files_scanned += 1;
            p.current_path = path.to_string_lossy().to_string();
            let _ = tx.send(ScanMessage::Progress(p.clone()));
        }
    }

    if node.is_dir {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let child_path = entry.path();
                if child_path.is_symlink() { continue; }
                
                let child_node = scan_recursive(&child_path, tx, progress);
                node.size += child_node.size;
                node.children.push(child_node);
            }
        }
        node.children.sort_by(|a, b| b.size.cmp(&a.size));
    } else {
        if let Ok(metadata) = path.metadata() {
            node.size = metadata.len();
            if let Ok(mut p) = progress.lock() {
                p.bytes_scanned += node.size;
            }
        }
    }
    node
}
