use std::collections::HashMap;

type FsFileId = usize;
const ROOT_FID: FsFileId = 0;

#[derive(Debug, PartialEq)]
pub enum FsFileType {
    Directory,
    File,
}

#[derive(Debug)]
pub struct FsFile {
    ftype: FsFileType,
    _name: String,
    parent: FsFileId,
    size: u64,
}

impl FsFile {
    fn is_directory(&self) -> bool {
        self.ftype == FsFileType::Directory
    }
}

type Metadata = HashMap<FsFileId, HashMap<String, FsFileId>>;
type Inodes = HashMap<FsFileId, FsFile>;

#[derive(Debug)]
pub struct FileSystem {
    free_id: FsFileId,
    metadata: Metadata,
    inodes: Inodes,
    pwd: FsFileId,
    total_size: u64,
}

impl FileSystem {
    pub fn new(total_size: u64) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert(ROOT_FID, HashMap::new());
        let mut inodes = HashMap::new();
        inodes.insert(
            ROOT_FID,
            FsFile {
                ftype: FsFileType::Directory,
                _name: "/".to_string(),
                parent: ROOT_FID,
                size: 0,
            },
        );
        FileSystem {
            free_id: ROOT_FID + 1,
            metadata,
            inodes,
            pwd: ROOT_FID,
            total_size,
        }
    }

    fn new_fsid(&mut self) -> FsFileId {
        let new = self.free_id;
        self.free_id += 1;
        new
    }

    pub fn create_dir(&mut self, name: String) -> FsFileId {
        let fsid = self.new_fsid();
        let dir = FsFile {
            ftype: FsFileType::Directory,
            _name: name.clone(),
            parent: self.pwd,
            size: 0,
        };
        self.inodes.insert(fsid, dir);
        self.metadata.insert(fsid, HashMap::new());
        self.metadata
            .get_mut(&self.pwd)
            .expect("pwd to have metadata")
            .insert(name, fsid);
        fsid
    }

    pub fn cd(&mut self, dir_name: String) {
        // Edge case: cd ..
        if dir_name == ".." {
            let pwd_inode = self.inodes.get(&self.pwd).expect("pwd to have an inode");
            self.pwd = pwd_inode.parent;
            return;
        }

        // Check if the directory is already listed
        let metadata = self
            .metadata
            .get(&self.pwd)
            .expect("the pwd to have an associated inode");
        if let Some(&child_fsid) = metadata.get(&dir_name) {
            let child_inode = self
                .inodes
                .get(&child_fsid)
                .expect("not to have dangling pointers");

            // Something of this name exists; check that it's a directory
            if !child_inode.is_directory() {
                panic!("Attempted to cd into a file.");
            }

            // It's a directory, and it already existed. Move the pwd there.
            self.pwd = child_fsid;
        } else {
            // The directory is not indexed.
            // Create it.
            let new_fsid = self.create_dir(dir_name);
            self.pwd = new_fsid;
        }
    }

    pub fn put_file(&mut self, name: String, size: u64) {
        let fsid = self.new_fsid();
        let file = FsFile {
            ftype: FsFileType::File,
            _name: name.clone(),
            parent: self.pwd,
            size,
        };
        self.inodes.insert(fsid, file);
        self.metadata
            .get_mut(&self.pwd)
            .expect("pwd to have a metadata entry")
            .insert(name, fsid);
    }

    pub fn finalize(mut self) -> Self {
        Self::measure_sizes(&self.metadata, &mut self.inodes, &ROOT_FID);
        self
    }
    
    pub fn occupied_size(&self) -> u64 {
        self.inodes.get(&ROOT_FID).expect("root to have inode").size
    }

    fn measure_sizes(metadata: &Metadata, inodes: &mut Inodes, root: &FsFileId) -> u64 {
        let mut size = 0;
        let child_ids = metadata.get(root).expect("metadata to exist").values();
        for child_id in child_ids {
            let child_inode = inodes.get(child_id).expect("child to have an inode");
            match child_inode.ftype {
                FsFileType::Directory => size += Self::measure_sizes(metadata, inodes, child_id),
                FsFileType::File => size += child_inode.size,
            }
        }
        inodes.get_mut(root).expect("inode to exist").size = size;
        size
    }

    pub fn total_size_below(&self, limit: u64) -> u64 {
        return self.total_size_below_recursive(limit, &ROOT_FID);
    }

    fn total_size_below_recursive(&self, limit: u64, root: &FsFileId) -> u64 {
        let inode = self.inodes.get(root).expect("root to have an inode");
        let mut total = 0;
        if inode.size <= limit {
            total = inode.size;
        }
        let child_ids = self.metadata.get(root).expect("metadata to exist").values();
        for child_id in child_ids {
            let child_inode = self.inodes.get(child_id).expect("child to have an inode");
            if child_inode.is_directory() {
                total += self.total_size_below_recursive(limit, child_id);
            }
        }
        return total;
    }

    pub fn delete_to_free(&self, to_free: u64) -> u64 {
        let occupied = self.occupied_size();
        let currently_free = self.total_size - occupied;
        if currently_free >= to_free {
            return 0;
        }
        let need = to_free - currently_free;
        println!("Need to free up {}", need);
        let to_delete = self
            .delete_to_free_recursive(need, &ROOT_FID)
            .expect("there to be an answer");
        self.inodes
            .get(&to_delete)
            .expect("directory to have an inode")
            .size
    }

    fn delete_to_free_recursive(&self, min_size: u64, root: &FsFileId) -> Option<FsFileId> {
        // Find the smallest child occupying at least the size we need to free up
        let mut best_candidate = None;
        let mut best_candidate_size = None;
        let child_ids = self.metadata.get(root).expect("metadata to exist").values();
        for child_id in child_ids {
            let child_inode = self.inodes.get(child_id).expect("child to have an inode");
            // We only consider directories that would free enough space.
            if !child_inode.is_directory() || child_inode.size < min_size {
                continue;
            }
            
            // See if there's a directory inside this one that's smaller but still suffices.
            let child_candidate = self.delete_to_free_recursive(min_size, child_id);
            
            // If no sub-directory satisfies, the parent directory does.
            let candidate = child_candidate.unwrap_or(*child_id);
            let candidate_inode = self.inodes.get(&candidate).expect("candidate to have an inode");

            if best_candidate.is_none() || candidate_inode.size < best_candidate_size.unwrap() {
                best_candidate = Some(candidate);
                best_candidate_size = Some(candidate_inode.size);
            }
        }
        
        best_candidate
    }
}
