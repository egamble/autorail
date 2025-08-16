use crate::common::{
  complete_function_str,
  create_and_writeln
};


const BUILD: &str =
  "***/signs/_build
***/stations/_build
***/switches/_build";

const STATION_BUILD_N: &str =
  r#"data merge block ~-1 ~1 ~ {front_text: {messages: [{"text":""},{"text":"Select Station","color":"dark_blue","click_event":{"action":"run_command","command":"***/select/_start {direction:n}"}},{"text":""},{"text":""}]}}

data merge block ~1 ~1 ~ {front_text: {messages: [{"text":""},{"text":"Destroy Carts","color":"dark_blue","click_event":{"action":"run_command","command":"***/x/station/destroy"}},{"text":""},{"text":""}]}}

$data merge block ~ ~1 ~ {front_text: {messages: [{"text":""},{"text":"Launch Cart","color":"dark_blue","click_event":{"action":"run_command","command":"***/x/station/launch/n {next_station_id:$(next_station_id)}"}},{"text":""},{"text":""}]}}

setblock ~1 ~2 ~ air
setblock ~-1 ~2 ~ air
setblock ~ ~-2 ~-1 air

$setblock ~ ~-2 ~-1 command_block[facing=down]{Command:"***/x/station/incoming {next_station_id:$(next_station_id)}"}"#;

const STATION_BUILD_S: &str =
  r#"data merge block ~1 ~1 ~ {front_text: {messages: [{"text":""},{"text":"Select Station","color":"dark_blue","click_event":{"action":"run_command","command":"***/select/_start {direction:s}"}},{"text":""},{"text":""}]}}

data merge block ~-1 ~1 ~ {front_text: {messages: [{"text":""},{"text":"Destroy Carts","color":"dark_blue","click_event":{"action":"run_command","command":"***/x/station/destroy"}},{"text":""},{"text":""}]}}

$data merge block ~ ~1 ~ {front_text: {messages: [{"text":""},{"text":"Launch Cart","color":"dark_blue","click_event":{"action":"run_command","command":"***/x/station/launch/s {next_station_id:$(next_station_id)}"}},{"text":""},{"text":""}]}}

setblock ~-1 ~2 ~ air
setblock ~1 ~2 ~ air
setblock ~ ~-2 ~1 air

$setblock ~ ~-2 ~1 command_block[facing=down]{Command:"***/x/station/incoming {next_station_id:$(next_station_id)}"}"#;

const STATION_BUILD_W: &str =
  r#"data merge block ~ ~1 ~1 {front_text: {messages: [{"text":""},{"text":"Select Station","color":"dark_blue","click_event":{"action":"run_command","command":"***/select/_start {direction:w}"}},{"text":""},{"text":""}]}}

data merge block ~ ~1 ~-1 {front_text: {messages: [{"text":""},{"text":"Destroy Carts","color":"dark_blue","click_event":{"action":"run_command","command":"***/x/station/destroy"}},{"text":""},{"text":""}]}}

$data merge block ~ ~1 ~ {front_text: {messages: [{"text":""},{"text":"Launch Cart","color":"dark_blue","click_event":{"action":"run_command","command":"***/x/station/launch/w {next_station_id:$(next_station_id)}"}},{"text":""},{"text":""}]}}

setblock ~ ~2 ~-1 air
setblock ~ ~2 ~1 air
setblock ~-1 ~-2 ~ air

$setblock ~-1 ~-2 ~ command_block[facing=down]{Command:"***/x/station/incoming {next_station_id:$(next_station_id)}"}"#;

const STATION_BUILD_E: &str =
  r#"data merge block ~ ~1 ~-1 {front_text: {messages: [{"text":""},{"text":"Select Station","color":"dark_blue","click_event":{"action":"run_command","command":"***/select/_start {direction:e}"}},{"text":""},{"text":""}]}}

data merge block ~ ~1 ~1 {front_text: {messages: [{"text":""},{"text":"Destroy Carts","color":"dark_blue","click_event":{"action":"run_command","command":"***/x/station/destroy"}},{"text":""},{"text":""}]}}

$data merge block ~ ~1 ~ {front_text: {messages: [{"text":""},{"text":"Launch Cart","color":"dark_blue","click_event":{"action":"run_command","command":"***/x/station/launch/e {next_station_id:$(next_station_id)}"}},{"text":""},{"text":""}]}}

setblock ~ ~2 ~1 air
setblock ~ ~2 ~-1 air
setblock ~1 ~-2 ~ air

$setblock ~1 ~-2 ~ command_block[facing=down]{Command:"***/x/station/incoming {next_station_id:$(next_station_id)}"}"#;

const STATION_NAME_SIGN: &str =
   "***/build
$***/x/teleport/s$(next_station_id) {elevation:-20}";

const STATION_BUILD_JUMP: &str =
   "***/build
$***/x/jump/s$(next_station_id)";

const STATION_DESTROY: &str =
  r#"kill @e[type=minecart,name=!"NoKill"]"#;

const STATION_INCOMING: &str =
  r#"$execute if entity @e[type=minecart,name="BuildAll",distance=..2] run ***/x/station/build_jump {next_station_id:$(next_station_id)}

kill @e[type=minecart,name=!"NoKill",name=!"BuildAll",distance=..2]"#;

const STATION_INCOMING_STRICT: &str =
  r#"kill @e[type=minecart,name=!"NoKill",distance=..2]

$data merge block ~ ~ ~ {Command:"***/x/station/incoming {next_station_id:$(next_station_id)}"}"#;

const STATION_OUTGOING: &str =
  r#"$execute if entity @e[type=minecart,name=!"BuildAll",distance=..2] run data merge block ~ ~ ~ {Command:"***/x/station/incoming {next_station_id:$(next_station_id)}"}

$execute if entity @e[type=minecart,name="BuildAll",distance=..2] run data merge block ~ ~ ~ {Command:"***/x/station/incoming_strict {next_station_id:$(next_station_id)}"}

$execute if entity @e[type=minecart,name="BuildAll",distance=..2] run ***/x/station/build_jump {next_station_id:$(next_station_id)}"#;

const STATION_QUICK_SELECT: &str =
  r#"$execute positioned $(x) $(y) $(z) run clone ~ ~ ~ ~ ~ ~ ~ ~1 ~

$execute positioned $(x) $(y) $(z) run ***/select/$(direction)/$(select_fn)"#;

const STATION_LAUNCH_N: &str =
  r#"$data merge block ~ ~-3 ~-1 {Command:"***/x/station/outgoing {next_station_id:$(next_station_id)}"}

data merge entity @e[type=minecart,distance=..1.5,limit=1] {Motion:[0.0,0.0,-1.0]}"#;

const STATION_LAUNCH_S: &str =
  r#"$data merge block ~ ~-3 ~1 {Command:"***/x/station/outgoing {next_station_id:$(next_station_id)}"}

data merge entity @e[type=minecart,distance=..1.5,limit=1] {Motion:[0.0,0.0,1.0]}"#;

const STATION_LAUNCH_W: &str =
  r#"$data merge block ~-1 ~-3 ~ {Command:"***/x/station/outgoing {next_station_id:$(next_station_id)}"}

data merge entity @e[type=minecart,distance=..1.5,limit=1] {Motion:[-1.0,0.0,0.0]}"#;

const STATION_LAUNCH_E: &str =
  r#"$data merge block ~1 ~-3 ~ {Command:"***/x/station/outgoing {next_station_id:$(next_station_id)}"}

data merge entity @e[type=minecart,distance=..1.5,limit=1] {Motion:[1.0,0.0,0.0]}"#;

const STATION_SUMMON_N: &str =
 r#"setblock ~ ~ ~ air
setblock ~2 ~ ~ air

data merge block ~ ~-1 ~ {front_text: {messages: [{"text":""},{"text":"Select Station","click_event":{"action":"run_command","command":"***/select/_start {direction:n}"},"color":"dark_blue"},{"text":""},{"text":""}]}}

$summon minecart ~1 ~-0.5 ~ {CustomName:"S$(station_id)"}"#;

const STATION_SUMMON_S: &str =
  r#"setblock ~ ~ ~ air
setblock ~-2 ~ ~ air

data merge block ~ ~-1 ~ {front_text: {messages: [{"text":""},{"text":"Select Station","click_event":{"action":"run_command","command":"***/select/_start {direction:s}"},"color":"dark_blue"},{"text":""},{"text":""}]}}

$summon minecart ~-1 ~-0.5 ~ {CustomName:"S$(station_id)"}"#;

const STATION_SUMMON_W: &str =
  r#"setblock ~ ~ ~ air
setblock ~ ~ ~-2 air

data merge block ~ ~-1 ~ {front_text: {messages: [{"text":""},{"text":"Select Station","click_event":{"action":"run_command","command":"***/select/_start {direction:w}"},"color":"dark_blue"},{"text":""},{"text":""}]}}

$summon minecart ~ ~-0.5 ~-1 {CustomName:"S$(station_id)"}"#;

const STATION_SUMMON_E: &str =
  r#"setblock ~ ~ ~ air
setblock ~ ~ ~2 air

data merge block ~ ~-1 ~ {front_text: {messages: [{"text":""},{"text":"Select Station","click_event":{"action":"run_command","command":"***/select/_start {direction:e}"},"color":"dark_blue"},{"text":""},{"text":""}]}}

$summon minecart ~ ~-0.5 ~1 {CustomName:"S$(station_id)"}"#;

const STATION_SUMMON_BUILD_ALL: &str =
  r#"$summon minecart $(x) $(y) $(z) {CustomName:"BuildAll"}"#;

const STATION_TELEPORT_N: &str =
  r#"setblock ~ ~ ~ air
setblock ~-2 ~ ~ air

data merge block ~-2 ~-1 ~ {front_text: {messages: [{"text":""},{"text":"Select Station","click_event":{"action":"run_command","command":"***/select/_start {direction:n}"},"color":"dark_blue"},{"text":""},{"text":""}]}}

$***/x/teleport/s$(station_id) {elevation:0}"#;

const STATION_TELEPORT_S: &str =
  r#"setblock ~ ~ ~ air
setblock ~2 ~ ~ air

data merge block ~2 ~-1 ~ {front_text: {messages: [{"text":""},{"text":"Select Station","click_event":{"action":"run_command","command":"***/select/_start {direction:s}"},"color":"dark_blue"},{"text":""},{"text":""}]}}

$***/x/teleport/s$(station_id) {elevation:0}"#;

const STATION_TELEPORT_W: &str =
  r#"setblock ~ ~ ~ air
setblock ~ ~ ~2 air

data merge block ~ ~-1 ~2 {front_text: {messages: [{"text":""},{"text":"Select Station","click_event":{"action":"run_command","command":"***/select/_start {direction:w}"},"color":"dark_blue"},{"text":""},{"text":""}]}}

$***/x/teleport/s$(station_id) {elevation:0}"#;

const STATION_TELEPORT_E: &str =
  r#"setblock ~ ~ ~ air
setblock ~ ~ ~-2 air

data merge block ~ ~-1 ~-2 {front_text: {messages: [{"text":""},{"text":"Select Station","click_event":{"action":"run_command","command":"***/select/_start {direction:e}"},"color":"dark_blue"},{"text":""},{"text":""}]}}

$***/x/teleport/s$(station_id) {elevation:0}"#;

const STATION_JUMP_N: &str =
  r#"$***/x/station/jump/$(dim) {x:$(x),y:$(y),z:$(z)}
$execute in $(dim) positioned $(x) $(y) $(z) run data merge entity @e[type=minecart,distance=..1,limit=1] {Motion:[0.0,0.0,-1.0]}"#;

const STATION_JUMP_S: &str =
  r#"$***/x/station/jump/$(dim) {x:$(x),y:$(y),z:$(z)}
$execute in $(dim) positioned $(x) $(y) $(z) run data merge entity @e[type=minecart,distance=..1,limit=1] {Motion:[0.0,0.0,1.0]}"#;

const STATION_JUMP_W: &str =
  r#"$***/x/station/jump/$(dim) {x:$(x),y:$(y),z:$(z)}
$execute in $(dim) positioned $(x) $(y) $(z) run data merge entity @e[type=minecart,distance=..1,limit=1] {Motion:[-1.0,0.0,0.0]}"#;

const STATION_JUMP_E: &str =
  r#"$***/x/station/jump/$(dim) {x:$(x),y:$(y),z:$(z)}
$execute in $(dim) positioned $(x) $(y) $(z) run data merge entity @e[type=minecart,distance=..1,limit=1] {Motion:[1.0,0.0,0.0]}"#;

const STATION_JUMP_OVERWORLD: &str =
  r#"$execute as @n[type=minecart] in overworld run tp @s $(x) $(y) $(z)
$execute in overworld run tp @p $(x) $(y) $(z)
execute in overworld at @p run ride @p mount @n[type=minecart]"#;

const STATION_JUMP_THE_NETHER: &str =
  r#"$execute as @n[type=minecart] in the_nether run tp @s $(x) $(y) $(z)
$execute in the_nether run tp @p $(x) $(y) $(z)
execute in the_nether at @p run ride @p mount @n[type=minecart]"#;

const STATION_JUMP_THE_END: &str =
  r#"$execute as @n[type=minecart] in the_end run tp @s $(x) $(y) $(z)
$execute in the_end run tp @p $(x) $(y) $(z)
execute in the_end at @p run ride @p mount @n[type=minecart]"#;


const SWITCH_SET_N_NW: &str = "setblock ~ ~2 ~2 rail[shape=north_west]";
const SWITCH_SET_N_NE: &str = "setblock ~ ~2 ~2 rail[shape=north_east]";
const SWITCH_SET_N_SW: &str = "setblock ~ ~2 ~2 rail[shape=south_west]";
const SWITCH_SET_N_SE: &str = "setblock ~ ~2 ~2 rail[shape=south_east]";

const SWITCH_SET_S_NW: &str = "setblock ~ ~2 ~-2 rail[shape=north_west]";
const SWITCH_SET_S_NE: &str = "setblock ~ ~2 ~-2 rail[shape=north_east]";
const SWITCH_SET_S_SW: &str = "setblock ~ ~2 ~-2 rail[shape=south_west]";
const SWITCH_SET_S_SE: &str = "setblock ~ ~2 ~-2 rail[shape=south_east]";

const SWITCH_SET_W_NW: &str = "setblock ~2 ~2 ~ rail[shape=north_west]";
const SWITCH_SET_W_NE: &str = "setblock ~2 ~2 ~ rail[shape=north_east]";
const SWITCH_SET_W_SW: &str = "setblock ~2 ~2 ~ rail[shape=south_west]";
const SWITCH_SET_W_SE: &str = "setblock ~2 ~2 ~ rail[shape=south_east]";

const SWITCH_SET_E_NW: &str = "setblock ~-2 ~2 ~ rail[shape=north_west]";
const SWITCH_SET_E_NE: &str = "setblock ~-2 ~2 ~ rail[shape=north_east]";
const SWITCH_SET_E_SW: &str = "setblock ~-2 ~2 ~ rail[shape=south_west]";
const SWITCH_SET_E_SE: &str = "setblock ~-2 ~2 ~ rail[shape=south_east]";


fn write_fixed_station_functions(out_path: &String) {
  create_and_writeln(
    &format!("{out_path}/x/station/build/n.mcfunction"),
    complete_function_str(STATION_BUILD_N)
  );
  
  create_and_writeln(
    &format!("{out_path}/x/station/build/s.mcfunction"),
    complete_function_str(STATION_BUILD_S)
  );
  
  create_and_writeln(
    &format!("{out_path}/x/station/build/w.mcfunction"),
    complete_function_str(STATION_BUILD_W)
  );
  
  create_and_writeln(
    &format!("{out_path}/x/station/build/e.mcfunction"),
    complete_function_str(STATION_BUILD_E)
  );
  
  create_and_writeln(
    &format!("{out_path}/x/station/launch/n.mcfunction"),
    complete_function_str(STATION_LAUNCH_N)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/launch/s.mcfunction"),
    complete_function_str(STATION_LAUNCH_S)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/launch/w.mcfunction"),
    complete_function_str(STATION_LAUNCH_W)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/launch/e.mcfunction"),
    complete_function_str(STATION_LAUNCH_E)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/name_sign.mcfunction"),
    complete_function_str(STATION_NAME_SIGN)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/build_jump.mcfunction"),
    complete_function_str(STATION_BUILD_JUMP)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/destroy.mcfunction"),
    complete_function_str(STATION_DESTROY)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/incoming.mcfunction"),
    complete_function_str(STATION_INCOMING)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/incoming_strict.mcfunction"),
    complete_function_str(STATION_INCOMING_STRICT)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/outgoing.mcfunction"),
    complete_function_str(STATION_OUTGOING)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/quick_select.mcfunction"),
    complete_function_str(STATION_QUICK_SELECT)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/summon/n.mcfunction"),
    complete_function_str(STATION_SUMMON_N)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/summon/s.mcfunction"),
    complete_function_str(STATION_SUMMON_S)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/summon/w.mcfunction"),
    complete_function_str(STATION_SUMMON_W)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/summon/e.mcfunction"),
    complete_function_str(STATION_SUMMON_E)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/summon_build_all.mcfunction"),
    complete_function_str(STATION_SUMMON_BUILD_ALL)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/teleport/n.mcfunction"),
    complete_function_str(STATION_TELEPORT_N)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/teleport/s.mcfunction"),
    complete_function_str(STATION_TELEPORT_S)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/teleport/w.mcfunction"),
    complete_function_str(STATION_TELEPORT_W)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/teleport/e.mcfunction"),
    complete_function_str(STATION_TELEPORT_E)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/jump/n.mcfunction"),
    complete_function_str(STATION_JUMP_N)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/jump/s.mcfunction"),
    complete_function_str(STATION_JUMP_S)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/jump/w.mcfunction"),
    complete_function_str(STATION_JUMP_W)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/jump/e.mcfunction"),
    complete_function_str(STATION_JUMP_E)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/jump/overworld.mcfunction"),
    complete_function_str(STATION_JUMP_OVERWORLD)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/jump/the_nether.mcfunction"),
    complete_function_str(STATION_JUMP_THE_NETHER)
  );

  create_and_writeln(
    &format!("{out_path}/x/station/jump/the_end.mcfunction"),
    complete_function_str(STATION_JUMP_THE_END)
  );
}


fn write_fixed_switch_functions(out_path: &String) {
  create_and_writeln(
    &format!("{out_path}/x/switch/set_n_nw.mcfunction"),
    complete_function_str(SWITCH_SET_N_NW)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_n_ne.mcfunction"),
    complete_function_str(SWITCH_SET_N_NE)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_n_sw.mcfunction"),
    complete_function_str(SWITCH_SET_N_SW)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_n_se.mcfunction"),
    complete_function_str(SWITCH_SET_N_SE)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_s_nw.mcfunction"),
    complete_function_str(SWITCH_SET_S_NW)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_s_ne.mcfunction"),
    complete_function_str(SWITCH_SET_S_NE)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_s_sw.mcfunction"),
    complete_function_str(SWITCH_SET_S_SW)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_s_se.mcfunction"),
    complete_function_str(SWITCH_SET_S_SE)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_w_nw.mcfunction"),
    complete_function_str(SWITCH_SET_W_NW)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_w_ne.mcfunction"),
    complete_function_str(SWITCH_SET_W_NE)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_w_sw.mcfunction"),
    complete_function_str(SWITCH_SET_W_SW)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_w_se.mcfunction"),
    complete_function_str(SWITCH_SET_W_SE)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_e_nw.mcfunction"),
    complete_function_str(SWITCH_SET_E_NW)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_e_ne.mcfunction"),
    complete_function_str(SWITCH_SET_E_NE)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_e_sw.mcfunction"),
    complete_function_str(SWITCH_SET_E_SW)
  );

  create_and_writeln(
    &format!("{out_path}/x/switch/set_e_se.mcfunction"),
    complete_function_str(SWITCH_SET_E_SE)
  );
}


pub fn write_fixed_functions(out_path: &String) {
  create_and_writeln(
    &format!("{out_path}/build.mcfunction"),
    complete_function_str(BUILD)
  );

  write_fixed_station_functions(out_path);

  write_fixed_switch_functions(out_path);
}
