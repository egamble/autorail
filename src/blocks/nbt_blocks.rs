use std::collections::HashMap;
use std::cmp;
use std::str;

use crate::common::{
  ChunkCoords,
  BlockCoords,
  Block,
  Realm,
  RailData,
  SignData,
  EMPTY
};
use crate::common::{
  block_name_to_id,
  facing_to_sign_data,
  shape_to_rail_data,
  read_i32
};

use crate::blocks::nbt::Nbt;
use crate::blocks::nbt::read_chunk_nbt;


pub fn find_chunk_nbt_blocks(chunk_coords: ChunkCoords, chunk_nbt: Vec<u8>) -> Vec<Block> {
  let mut blocks = Vec::new();

  if let Nbt::NbtCompound(chunk_root) = read_chunk_nbt(&chunk_nbt) {
    let sign_text_map = make_sign_text_map(&chunk_root, chunk_coords);

    if let Some(sections_tag) = chunk_root.get("sections") {
      if let Nbt::NbtList(sections) = sections_tag {
        for section_tag in sections {
          if let Nbt::NbtCompound(section) = section_tag {
            if let Some(block_states_tag) = section.get("block_states") {
              if let Nbt::NbtCompound(block_states) = block_states_tag {
                if let Some(palette_tag) = block_states.get("palette") {
                  if let Nbt::NbtList(palette) = palette_tag {
                    if palette.len() > 0 {
                      if let Some(block_states_data_tag) = block_states.get("data") {
                        if let Nbt::NbtData(block_states_raw_data) = block_states_data_tag {
                          if let Some(section_y_tag) = section.get("Y") {
                            if let Nbt::NbtData(section_y_data) = section_y_tag {
                              let section_y = section_y_data[0] as i8;

                              let palette_index_width = cmp::max(4,
                                                                 palette.len()
                                                                 .next_power_of_two()
                                                                 .ilog2() as usize);
                              let indices_per_data_element = (64 as usize).div_euclid(palette_index_width);
                              
                              let palette_map = make_palette_map(palette);

                              for block_index in 0..4096 {
                                let palette_index = get_palette_index(block_index,
                                                                      block_states_raw_data,
                                                                      palette_index_width,
                                                                      indices_per_data_element);

                                if let Some(template_block) = palette_map.get(&palette_index) {
                                  let block_coords = block_index_to_block_coords(block_index, chunk_coords, section_y);

                                  let block = Block {
                                    id: template_block.id,
                                    coords: block_coords,
                                    rail_data: template_block.rail_data,
                                    sign_data: template_block.sign_data,
                                    sign_text: match sign_text_map.get(&block_coords) {
                                      Some(sign_text) => sign_text.to_string(),
                                      None => EMPTY
                                    }
                                  };

                                  blocks.push(block);
                                }
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }

  blocks
}


fn block_index_to_block_coords(block_index: usize, chunk_coords: ChunkCoords, section_y: i8) -> BlockCoords {
  let (chunk_x, chunk_z, realm) = chunk_coords;

  let block_y_offset = (block_index as i32).div_euclid(256);
  let xz_rem = (block_index as i32).rem_euclid(256);

  let block_z_offset = xz_rem.div_euclid(16);
  let block_x_offset = xz_rem.rem_euclid(16);
  
  (
    block_x_offset + 16 * chunk_x,
    block_y_offset + 16 * (section_y as i32),
    block_z_offset + 16 * chunk_z,
    realm
  )
}


fn get_palette_index(block_index: usize,
                     block_states_raw_data: &Vec<u8>,
                     palette_index_width: usize,
                     indices_per_data_element: usize) -> usize {
  let data_element_start = 8 * block_index.div_euclid(indices_per_data_element);
  let start_bit = palette_index_width * block_index.rem_euclid(indices_per_data_element);

  let data_element = usize::from_be_bytes( block_states_raw_data[data_element_start..data_element_start+8]
                                         .try_into()
                                         .unwrap() );

  data_element
    .checked_shl((64 - (start_bit + palette_index_width)) as u32)
    .unwrap()
    .checked_shr((64 - palette_index_width) as u32)
    .unwrap()
}


fn make_palette_map(palette: &Vec<Nbt>) -> HashMap<usize, Block> {
  let mut palette_map = HashMap::new();

  for palette_index in 0..palette.len() {
    let palette_tag = &palette[palette_index];
    if let Nbt::NbtCompound(palette) = palette_tag {
    
      if let Some(name_tag) = palette.get("Name") {
        if let Nbt::NbtData(name_data) = name_tag {
          let name: &str = str::from_utf8(name_data).unwrap();

          if let Some(block_id) = block_name_to_id(name) {
            if let Some(properties_tag) = palette.get("Properties") {
              if let Nbt::NbtCompound(properties) = properties_tag {

                let mut rail_data: RailData = RailData::NS;
                let mut sign_data: SignData = SignData::N;

                if (&block_id).is_rail_id() {
                  if let Some(shape_tag) = properties.get("shape") {
                    if let Nbt::NbtData(shape_data) = shape_tag {
                      let shape: &str = str::from_utf8(shape_data).unwrap();
                      if let Some(bd) = shape_to_rail_data(shape) {
                        rail_data = bd;
                      }
                    }
                  }
                }
                if (&block_id).is_sign_id() {
                  if let Some(facing_tag) = properties.get("facing") {
                    if let Nbt::NbtData(facing_data) = facing_tag {
                      let facing: &str = str::from_utf8(facing_data).unwrap();
                      if let Some(bd) = facing_to_sign_data(facing) {
                        sign_data = bd;
                      }
                    }
                  }
                }

                let template_block = Block {
                  id: block_id,
                  coords: (0, 0, 0, Realm::Overworld),
                  rail_data: rail_data,
                  sign_data: sign_data,
                  sign_text: EMPTY,
                };
                palette_map.insert(palette_index, template_block);
              }
            }
          }
        }
      }
    }
  }
  
  palette_map
}


fn make_sign_text_map(chunk_root: &HashMap<String, Nbt>, chunk_coords: ChunkCoords) -> HashMap<BlockCoords, String> {
  let (_, _, realm) = chunk_coords;

  let mut sign_text_map = HashMap::new();

  if let Some(block_entities_tag) = chunk_root.get("block_entities") {
    if let Nbt::NbtList(block_entities) = block_entities_tag {
      for block_entity_tag in block_entities {
        if let Nbt::NbtCompound(block_entity) = block_entity_tag {
          if let Some(block_id_tag) = block_entity.get("id") {
            if let Nbt::NbtData(block_id_data) = block_id_tag {
              let block_id: &str = str::from_utf8(block_id_data).unwrap();      
              if block_id == "minecraft:sign" {

                if let Some(block_x_tag) = block_entity.get("x") {
                  if let Nbt::NbtData(block_x_data) = block_x_tag {
                    let block_x = read_i32(block_x_data);
                    
                    if let Some(block_y_tag) = block_entity.get("y") {
                      if let Nbt::NbtData(block_y_data) = block_y_tag {
                        let block_y = read_i32(block_y_data);

                        if let Some(block_z_tag) = block_entity.get("z") {
                          if let Nbt::NbtData(block_z_data) = block_z_tag {
                            let block_z = read_i32(block_z_data);

                            let block_coords = (block_x, block_y, block_z, realm);
                            sign_text_map.insert(block_coords, extract_sign_text(block_entity));
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }

  sign_text_map
}


fn extract_sign_text(block_entity: &HashMap<String, Nbt>) -> String {
  let mut sign_text = EMPTY;

  if let Some(front_text_tag) = block_entity.get("front_text") {
    if let Nbt::NbtCompound(front_text) = front_text_tag {

      if let Some(messages_tag) = front_text.get("messages") {
        if let Nbt::NbtList(messages) = messages_tag {

          for message_index in 0..messages.len() {
            let message_tag = &messages[message_index];
            if let Nbt::NbtData(message_data) = message_tag {
              let message = str::from_utf8(message_data).unwrap();

              let mut next_text = text_from_json(message);
              next_text = next_text.trim().to_string();

              if sign_text != "" && next_text != "" {
                sign_text.push_str(" ");
              }
              sign_text.push_str(&next_text);
            }
          }
        }
      }
    }
  }

  sign_text
}


fn text_from_json(json: &str) -> String {
  let val: serde_json::Value = serde_json::from_str(json).unwrap();

  match val {
    serde_json::Value::String(text) => text,
    serde_json::Value::Object(object) => {
      match object.get("text") {
        Some(text_val) => match text_val {
          serde_json::Value::String(text) => text.to_string(),
          _ => EMPTY
        }
        _ => EMPTY
      }
    },
      _ => EMPTY
  }
}

