#![feature(concat_idents)]
#![feature(proc_macro_hygiene)]
#![feature(asm)]

use smash::lib::lua_const::*;
use smash::app::lua_bind::*;
use smash::app::*;
use smash::app::FighterManager;

static mut NOTIFY_LOG_EVENT_COLLISION_HIT_OFFSET : usize = 0x675A20;

mod ryu;
use crate::ryu::{SECRET_SENSATION, OPPONENT_X, OPPONENT_Y, CAMERA}; // Imports some of Ryu's variables into lib.rs

#[skyline::hook(offset = NOTIFY_LOG_EVENT_COLLISION_HIT_OFFSET)]
pub unsafe fn notify_log_event_collision_hit_replace(
fighter_manager: &mut FighterManager,
attacker_object_id: u32,
defender_object_id: u32, 
move_type: f32,
arg5: i32,
move_type_again: bool) -> u64 {
    let attacker_boma = sv_battle_object::module_accessor(attacker_object_id);
    let defender_boma = sv_battle_object::module_accessor(defender_object_id);
    // let attacker_fighter_kind = sv_battle_object::kind(attacker_object_id);
    let defender_fighter_kind = sv_battle_object::kind(defender_object_id);
    // let a_entry_id = WorkModule::get_int(attacker_boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    let d_entry_id = WorkModule::get_int(defender_boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    if defender_fighter_kind == *FIGHTER_KIND_RYU {
        if (MotionModule::motion_kind(defender_boma) == smash::hash40("appeal_hi_r") // Checks if Ryu's doing up taunt and it's the first 30 frames of the animation.
        || MotionModule::motion_kind(defender_boma) == smash::hash40("appeal_hi_l"))
        && MotionModule::frame(defender_boma) <= 30.0 {
            if utility::get_category(&mut *attacker_boma) == *BATTLE_OBJECT_CATEGORY_FIGHTER // Grabs the attacker's position and stores it in a public variable.
            || utility::get_category(&mut *attacker_boma) == *BATTLE_OBJECT_CATEGORY_ENEMY {
                OPPONENT_X[d_entry_id] = PostureModule::pos_x(attacker_boma); // Sets the variable to True, so Ryu's mod.rs can see it an start working.
                OPPONENT_Y[d_entry_id] = PostureModule::pos_y(attacker_boma);
                SECRET_SENSATION[d_entry_id] = true;
            }
            else if utility::get_category(&mut *attacker_boma) == *BATTLE_OBJECT_CATEGORY_WEAPON {
                let oboma = smash::app::sv_battle_object::module_accessor((WorkModule::get_int(attacker_boma, *WEAPON_INSTANCE_WORK_ID_INT_LINK_OWNER)) as u32);
                if utility::get_category(&mut *oboma) != *BATTLE_OBJECT_CATEGORY_FIGHTER { // Checks to see if the owner of what hit you is a Fighter or not
                    OPPONENT_X[d_entry_id] = PostureModule::pos_x(defender_boma); // If yes, stores the opponent's position
                    OPPONENT_Y[d_entry_id] = PostureModule::pos_y(defender_boma);
                }
                else {
                    OPPONENT_X[d_entry_id] = PostureModule::pos_x(oboma); // If no, stores Ryu's position (check Ryu's mod.rs for an explanation)
                    OPPONENT_Y[d_entry_id] = PostureModule::pos_y(oboma);
                }
                SECRET_SENSATION[d_entry_id] = true;
            }
            else {
                OPPONENT_X[d_entry_id] = PostureModule::pos_x(defender_boma); // If what his you is anything else, stores Ryu's position (for the same reason as above, will explain)
                OPPONENT_Y[d_entry_id] = PostureModule::pos_y(defender_boma);
                SECRET_SENSATION[d_entry_id] = true;
            }
        }
    }
    original!()(fighter_manager, attacker_object_id, defender_object_id, move_type, arg5, move_type_again)
}

#[skyline::hook(replace = smash::app::lua_bind::WorkModule::is_enable_transition_term )]
pub unsafe fn is_enable_transition_term_replace(module_accessor: &mut BattleObjectModuleAccessor, term: i32) -> bool {
    let fighter_kind = smash::app::utility::get_kind(module_accessor);
    let ret = original!()(module_accessor,term);
    let entry_id = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    
    if fighter_kind == *FIGHTER_KIND_RYU && entry_id < 8 { // ***Theoretically*** disables any inputs from Ryu as he's Ultra Instincting.
        if CAMERA[entry_id] {
            return false;
        }
        else {
            return ret;
        }
    }
    else {
        return ret;
    }
}

#[skyline::main(name = "secret sensation")]
pub fn main() {
    skyline::install_hook!(notify_log_event_collision_hit_replace);
    skyline::install_hook!(is_enable_transition_term_replace);
    ryu::install();
}