use std::str;
use std::collections::HashMap;

use crate::common::{
  read_two_byte_size,
  read_four_byte_size
};


pub enum Nbt {
  NbtCompound(HashMap<String, Nbt>),
  NbtList(Vec<Nbt>),
  
  // NbtData doesn't preserve tag IDs because we already know
  // what they must be based on the known chunk NBT structure.
  NbtData(Vec<u8>)
}


pub fn read_chunk_nbt(data: &Vec<u8>) -> Nbt {
  let (_, _, payload_start_index) = read_nbt_tag(data, 0);
  let (nbt_compound, _) = read_nbt_compound(data, payload_start_index);

  return nbt_compound;
}


fn read_nbt_compound(data: &Vec<u8>, payload_start_index: usize) -> (Nbt, usize) {
  let mut nbt_compound_map: HashMap<String, Nbt> = HashMap::new();

  let mut next_start_index = payload_start_index;

  loop {
    let (id, name, child_payload_start_index) = read_nbt_tag(data, next_start_index);

    let (child_option, next_index) = get_nbt_child(id, data, child_payload_start_index);
    if let Some(child) = child_option {
      next_start_index = next_index;
      nbt_compound_map.insert(name, child);
    } else {
      return (Nbt::NbtCompound(nbt_compound_map), child_payload_start_index);
    }
  }
}


fn read_nbt_list(data: &Vec<u8>, payload_start_index: usize) -> (Nbt, usize) {
  let mut nbt_list: Vec<Nbt> = Vec::new();

  let id = data[payload_start_index];
  let nbt_list_length = read_four_byte_size(data, payload_start_index + 1);
  
  let mut next_start_index = payload_start_index + 5;

  for nbt_list_index in 0..nbt_list_length {
    let (child_option, next_index) = get_nbt_child(id, data, next_start_index);
    if let Some(child) = child_option {
      next_start_index = next_index;
      nbt_list.insert(nbt_list_index, child);
    }
  }

  (Nbt::NbtList(nbt_list), next_start_index)
}


fn read_nbt_tag(data: &Vec<u8>, index: usize) -> (u8, String, usize) {
  let tag_id = data[index];

  if tag_id == 0 {
    return (0, "".to_string(), index + 1);
  }
  
  let name_length = read_two_byte_size(data, index + 1);
  let name_index = index + 3;
  let payload_start_index = name_index + name_length;

  let name: &str;
  if let Ok(n) = str::from_utf8(&data[name_index..payload_start_index]) {
    name = n;
  } else {
    name = ""
  }

  (tag_id, name.to_string(), payload_start_index)
}


fn get_nbt_child(id: u8, data: &Vec<u8>, index: usize) -> (Option<Nbt>, usize) {
    if let Some((nbt_data_length, nbt_data_offset)) = get_nbt_data_length_and_offset(id, data, index) {

      let nbt_data_start = index + nbt_data_offset;
      let next_index = nbt_data_start + nbt_data_length;

      (Some(Nbt::NbtData(data[nbt_data_start..next_index].to_vec())), next_index)

    } else {

      match id {
        9 => { // List
          let (child, next_index) = read_nbt_list(data, index);
          (Some(child), next_index)
        },
        10 => { // Compound
          let (child, next_index) = read_nbt_compound(data, index);
          (Some(child), next_index)
        },
        _ => { // all other IDs
          (None, 0)
        }
      }
    }
}


fn get_nbt_data_length_and_offset(id: u8, data: &Vec<u8>, index: usize) -> Option<(usize, usize)> {
   match id {
     1 => { // Byte
       Some((1, 0))
     },
     2 => { // Short
       Some((2, 0))
     },
     3 => { // Int
       Some((4, 0))
     },
     4 => { // Long
       Some((8, 0))
     },
     5 => { // Float
       Some((4, 0))
     },
     6 => { // Double
       Some((8, 0))
     },
     7 => { // Byte array
       Some((read_four_byte_size(data, index), 4))
     },
     8 => { // String
       Some((read_two_byte_size(data, index), 2))
     },
     11 => { // Int array
       Some((4 * read_four_byte_size(data, index), 4))
     },
     12 => { // Long array
       Some((8 * read_four_byte_size(data, index), 4))
     },
     _ => { // End (0), List (9), Compound (10) or anything else
       None
     }
   }
}
