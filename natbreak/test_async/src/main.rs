// #![feature(core_intrinsics)]
// #![feature(layout_for_ptr)]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
extern crate core;

use std::future;
use std::ptr::slice_from_raw_parts;
use std::sync::RwLock;
use std::thread::sleep;
use std::time::Duration;

#[derive(
Clone,
Copy,
Default,
Eq,
PartialEq,
Hash,

)]
pub struct Pubkey(pub(crate) [u8; 32]);
pub type SecondaryReverseIndexEntry = RwLock<Vec<Pubkey>>;

impl From<&[u8]> for Pubkey {
    fn from(data : &[u8]) -> Self {
        // let valid_value : [u8; 32] = data.try_into().unwrap();
        Self(
            <[u8; 32]>::try_from(<&[u8]>::clone(&data))
                .expect("Slice must be the same length as a Pubkey"),
        )
    }
}

#[tokio::main]
async fn main() {
    loop {
        let value = hello_world();
    }

}


async fn hello_world() -> String {
    sleep(Duration::from_secs(1));
    format!("hello world!")
}

pub mod tests {
    use std::collections::HashMap;
    // use std::intrinsics::size_of_val;
    use std::thread::sleep;
    use std::time::Duration;
    use std::mem;
    use std::mem::{size_of, size_of_val};
    use std::ptr::NonNull;
    use dashmap::DashMap;
    use rand::distributions::Alphanumeric;
    use rand::{Rng, thread_rng};
    use crate::{Pubkey, SecondaryReverseIndexEntry};

    #[test]
    fn empty_vec_spend_space() {
        let mut vec = Vec::new();
        for i in 1..120000000 {
            let empty_vec :Vec<u8> = Vec::new();
            vec.push(empty_vec);
        }
        println!("sleep");
        sleep(Duration::from_secs(30));
        println!("vec len {} ", vec.len())
    }

    #[test]
    fn empty_struct_spend_space() {
        // let empty_vec = Vec::new();
        println!("empty_vec spend space {}", size_of::<Vec<u8>>());
        println!("empty_vec spend space {}", size_of::<DashMap<Pubkey, u8>>());
        println!("empty_vec spend space {}", size_of::<SecondaryReverseIndexEntry>());
        println!("empty_vec spend space {}", size_of::<()>());
    }

    pub struct Bucket<T> {
        // Actually it is pointer to next element than element itself
        // this is needed to maintain pointer arithmetic invariants
        // keeping direct pointer to element introduces difficulty.
        // Using `NonNull` for variance and niche layout
        ptr: NonNull<T>,
    }

    #[test]
    fn hash_map_spend_space() {
        let _profiler = dhat::Profiler::builder().testing().build();
        println!("hash map bucket len {}", size_of::<Bucket<i64>>());
        let mut map = HashMap::new();
        println!("hash map bucket len {}", size_of::<Bucket<i32>>());
        println!("hash map bucket len {}", size_of::<Bucket<i32>>());
        for i in 0..1000000 {
            let mut rand_string: Vec<u8> = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .collect();

            // println!("{:?}", rand_string);
            let new_key = Pubkey::from(rand_string.as_slice());
            map.insert(new_key, new_key);
        }

        let stats = dhat::HeapStats::get();
        println!("stats curr_bytes {} ", stats.curr_bytes);
        println!("stats total_bytes {} ", stats.total_bytes);
        println!("sleep");
        // sleep(Duration::from_secs(1));
        println!("map size {} ", map.len());
        println!("map cap {} ", map.capacity());


        unsafe {
            println!("intrinsics map size {} ", size_of_val(&map));
        }

        println!("pubkey len {}", size_of::<Pubkey>());
        println!("pubkey len {}", size_of::<Vec<bool>>());
        println!("pubkey len {}", size_of::<[u8;32]>());

        // sleep(Duration::from_secs(10));
    }
}