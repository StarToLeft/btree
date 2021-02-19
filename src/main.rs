use std::cmp::{Eq, Ord};

// Leafs are always on the same level
// The tree grows upward, by splitting nodes

#[derive(Debug, Clone, Copy)]
pub struct Entry<K, P>
where
    K: Eq + Ord + Copy,
    P: Copy,
{
    key: K,
    value: P,
}

impl<K, P> Entry<K, P>
where
    K: Eq + Ord + Copy,
    P: Copy,
{
    pub fn new(key: K, value: P) -> Entry<K, P> {
        Entry { key, value }
    }

    pub fn get_key(&self) -> &K {
        &self.key
    }
}

#[derive(Debug)]
pub struct Node<K, P>
where
    K: Eq + Ord + Copy,
    P: Copy,
{
    pub t: usize,
    pub n: usize,
    pub leaf: bool,
    pub keys: Box<[Option<Entry<K, P>>]>,
    pub child: Box<[Option<Node<K, P>>]>,
}

impl<K, P> Node<K, P>
where
    K: Eq + Ord + Copy,
    P: Copy,
{
    pub fn new(t: usize, leaf: bool) -> Node<K, P> {
        // Initialize child
        let mut child = Vec::with_capacity(2 * t);
        unsafe {
            for _ in 0..child.capacity() {
                child.push(None);
            }
            child.set_len(child.capacity());
        }
        let child = child.into_boxed_slice();

        // Initialize keys
        let mut keys = Vec::with_capacity(2 * t - 1);
        unsafe {
            keys.set_len(keys.capacity());
            for i in 0..2 * t - 1 {
                keys[i] = None;
            }
        }
        let keys = keys.into_boxed_slice();

        Node {
            t,
            n: 0,
            leaf,
            keys,
            child,
        }
    }

    pub fn traverse<'a>(&'a self, t: &mut Vec<&'a Entry<K, P>>) {
        for i in 0..self.n {
            let k = self.keys[i].as_ref().unwrap();
            t.push(k);
        }

        if !self.leaf {
            for i in 0..self.n + 1 {
                let c = self.child[i].as_ref().unwrap();
                c.traverse(t);
            }
        }
    }

    pub fn search(&self, key: &K, force_linear: bool) -> Option<Entry<K, P>> {
        let mut i = 0;
        if !force_linear && self.n > 512 {
            let l = self.binary_search_keys(key);
            if l == -1 {
                return self.search(key, true);
            } else {
                i = l as usize;
            }
        } else {
            while i < self.n && self.keys[i].as_ref().unwrap().get_key() < key {
                i += 1;
            }
        }

        if self.n > i && self.keys[i].as_ref().unwrap().get_key() == key {
            return self.keys[i];
        }

        if i >= self.child.len() || self.leaf {
            return None;
        }

        self.child[i].as_ref().unwrap().search(key, force_linear)
    }

    pub fn binary_search_keys(&self, key: &K) -> isize {
        let mut low = 0;
        let mut high = self.n as isize - 1;

        while low <= high {
            let mid = low + ((high - low) / 2);

            if self.keys[mid as usize].as_ref().unwrap().get_key() == key {
                return mid;
            }

            if key < self.keys[mid as usize].as_ref().unwrap().get_key() {
                high = mid - 1;
            } else {
                low = mid + 1;
            }
        }

        -1
    }

    pub fn insert_non_full(&mut self, key: K, pointer: P) {
        let mut i: isize = (self.n - 1) as isize;

        // Insert into leaf if node is a leaf
        if self.leaf {
            while i >= 0 && self.keys[i as usize].as_ref().unwrap().get_key() > &key {
                self.keys[(i + 1) as usize] = self.keys[i as usize].take();
                i -= 1;
            }

            self.keys[(i + 1) as usize] = Some(Entry::new(key, pointer));
            self.n += 1;
        } else {
            while i >= 0 && self.keys[i as usize].as_ref().unwrap().get_key() > &key {
                i -= 1;
            }

            if self.child[(i + 1) as usize].as_ref().unwrap().n == 2 * self.t - 1 {
                self.split_nodes((i + 1) as usize, (i + 1) as usize);

                if self.keys[i as usize].as_ref().unwrap().get_key() < &key {
                    i += 1;
                }
            }
            self.child[(i + 1) as usize]
                .as_mut()
                .unwrap()
                .insert_non_full(key, pointer);
        }
    }

    pub fn split_nodes(&mut self, pos: usize, child_index: usize) {
        let y = self.child[child_index].as_mut().unwrap();

        // Create second node to take a piece of Ys keys
        let mut z = Node::new(self.t, y.leaf);
        z.n = y.t - 1;

        // Move [t - 1] keys from y to z, as we are splitting
        let mut j = 0;
        while j < self.t - 1 {
            z.keys[j] = y.keys[j + self.t].take();
            j += 1;
        }

        // When splitting, if the node is not a leaf, it has to move [t] children from y to z
        if !y.leaf {
            let mut j = 0;
            while j < self.t {
                z.child[j] = y.child[j + self.t].take();
                j += 1;
            }
        }

        y.n = self.t - 1;
        let c = y.keys[self.t - 1].take();

        let mut j = self.n;
        while j >= pos + 1 {
            self.child[j + 1] = self.child[j].take();
            j -= 1;
        }

        self.child[pos + 1] = Some(z);

        let mut j: isize = self.n as isize - 1;
        while j >= pos as isize {
            self.keys[(j + 1) as usize] = self.keys[j as usize].take();
            j -= 1;
        }

        self.keys[pos] = c;
        self.n += 1;
    }
}

#[derive(Debug)]
pub struct BTree<K, P>
where
    K: Eq + Ord + Copy,
    P: Copy,
{
    pub(crate) root: Option<Node<K, P>>,
    t: usize,
}

impl<K, P> BTree<K, P>
where
    K: Eq + Ord + Copy,
    P: Copy,
{
    pub fn new(t: usize) -> BTree<K, P> {
        if t < 2 {
            panic!("Degree may not be smaller than 2");
        }

        BTree { root: None, t }
    }

    pub fn traverse<'a>(&'a self) -> Option<Vec<&'a Entry<K, P>>> {
        let mut t = Vec::new();

        match &self.root {
            Some(r) => {
                r.traverse(&mut t);
                Some(t)
            }
            None => None,
        }
    }

    pub fn search(&self, key: &K) -> Option<Entry<K, P>> {
        match &self.root {
            Some(r) => r.search(&key, false),
            None => None,
        }
    }

    pub fn search_linear(&self, key: &K) -> Option<Entry<K, P>> {
        match &self.root {
            Some(r) => r.search(&key, true),
            None => None,
        }
    }

    pub fn insert(&mut self, key: K, pointer: P) {
        // Initialize new root if it doesn't already exist
        // Insert directly into it if it's new
        if self.root.is_none() {
            let mut root = Node::new(self.t, true);
            root.keys[0] = Some(Entry::new(key, pointer));
            root.n = 1;
            self.root = Some(root);
        } else {
            // Check if root is full
            if self.root.as_ref().unwrap().n == 2 * self.t - 1 {
                // Initialize a new root, prepare for a split
                let mut s = Node::new(self.t, false);

                // Steal the root and set it as a child of the new root,
                s.child[0] = self.root.take();

                // Split the old root, by the child of index 0
                s.split_nodes(0, 0);

                // The new root now contains two child, choose which one to insert into
                let mut index = 0;
                if s.keys[0].as_ref().unwrap().get_key() < &key {
                    index += 1;
                }
                s.child[index]
                    .as_mut()
                    .unwrap()
                    .insert_non_full(key, pointer);

                // Set new root
                self.root = Some(s);
            } else {
                // Insert into root if it's not full
                self.root.as_mut().unwrap().insert_non_full(key, pointer);
            }
        }
    }
}

fn main() {
    let full_time_t = std::time::Instant::now();
    let mut tree: BTree<_, _> = BTree::new(2056);

    for i in 0..1000000 {
        tree.insert(i, i);
    }
    let el = full_time_t.elapsed();
    println!("Insert time {:?}", el);

    let t = std::time::Instant::now();
    for i in 0..1000000 {
        tree.search(&i);
    }
    let el = t.elapsed();
    println!("Read time: {:?}", el);

    let el = full_time_t.elapsed();
    println!("Full-runtime - elapsed: {:?}", el);
}
