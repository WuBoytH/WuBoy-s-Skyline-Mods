use smash::phx::Hash40;
use smash::lua2cpp::L2CAgentBase;
use smash::lib::lua_const::*;
use smash::app::lua_bind::*;
use smash::app::FighterManager;
use smash::app::*;
use smash::lua2cpp::{L2CFighterCommon, L2CFighterBase};
use smash::phx::Vector2f;
use acmd::*;

// Converted to ACMD by DECEPTICON

pub static mut SECRET_SENSATION : [bool; 8] = [false; 8];
pub static mut CAMERA : [bool; 8] = [false; 8];
pub static mut OPPONENT_X : [f32; 8] = [0.0; 8];
pub static mut OPPONENT_Y : [f32; 8] = [0.0; 8];
static mut RYU_X : [f32; 8] = [0.0; 8];
static mut RYU_Y : [f32; 8] = [0.0; 8];
static mut SEC_SEN_TIMER : [f32; 8] = [-0.4; 8]; // I start this as -0.4 so that Ryu doesn't immediately start dodging, there's a little pause before he does
static mut OPPONENT_DIRECTION : [f32; 8] = [12.0; 8];
static mut VERT_EXTRA : [f32; 8] = [12.0; 8];

pub fn once_per_fighter_frame(fighter : &mut L2CFighterCommon) {
    unsafe {
        let module_accessor = smash::app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
        let entry_id = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
        let fighter_kind = smash::app::utility::get_kind(module_accessor);
        let status_kind = smash::app::lua_bind::StatusModule::status_kind(module_accessor);
        let lua_state = fighter.lua_state_agent; 
       
        
        if fighter_kind == *FIGHTER_KIND_RYU {
            if entry_id < 8 {
                
                if (MotionModule::motion_kind(module_accessor) == smash::hash40("appeal_hi_r") // Sets the damage Ryu takes to (effectively) 0 during the first 30 frames of up taunt.
                || MotionModule::motion_kind(module_accessor) == smash::hash40("appeal_hi_l"))
                && MotionModule::frame(module_accessor) <= 30.0 {
                    DamageModule::set_damage_mul(module_accessor, 0.000001);
                }
                else { // Resets his damage multiplier back to 1.0. If there's anything that also changes how much damage Ryu takes, however, then it will break that atm, but it's fine for items off at the moment.
                    DamageModule::set_damage_mul(module_accessor, 1.0);
                }

                // Secret Sensation???

                if SECRET_SENSATION[entry_id] {
                    if CAMERA[entry_id] == false { // Exists so all of this code will only happen once.
                        KineticModule::change_kinetic(module_accessor, *FIGHTER_KINETIC_TYPE_RESET); // Resets all knockback
                        StatusModule::change_status_request_from_script(module_accessor, *FIGHTER_STATUS_KIND_WAIT, true); // Sets Ryu into the idle state
                        acmd!(lua_state,{
                            CAM_ZOOM_IN_arg5(5.0, 0.0, 1.1, 0.0, 0.0) // Sets the camera
                            SLOW_OPPONENT(100.0, 32.0) // Slows the opponent down by 100x for 32 frames
                        });
                        SlowModule::set_whole(module_accessor, 4, 0); // Slows ***everything*** down by a 4x. This includes the above slowdown, which probably means I should shorten the above length of time but eh
                        JostleModule::set_status(module_accessor, false); // It *should* turn off body blocking for Ryu but I'm not sure, I couldn't tell
                        MotionModule::change_motion_inherit_frame_keep_rate(module_accessor,Hash40::new("turn"),0.0,0.0,0.0); // doesn't actually work, but is supposed to change his animation to the turn animation when switching directions
                        RYU_X[entry_id] = PostureModule::pos_x(boma); // Gets Ryu's position
                        RYU_Y[entry_id] = PostureModule::pos_y(boma);
                        if RYU_X[entry_id] < OPPONENT_X[entry_id] { // Checks where Ryu and his Opponent are relative to each other, and sets a value so Ryu always moves *behind* the opponent
                            OPPONENT_DIRECTION[entry_id] = 12.0;
                        }
                        else {
                            OPPONENT_DIRECTION[entry_id] = -12.0;
                        }
                        CAMERA[entry_id] = true; // Again, ensures that the above code only runs once.
                    }
                    if SEC_SEN_TIMER[entry_id] >= 0.0 { // This whole if statement is for linearly interpolating Ryu's position, instead of just teleporting him behind the opponent.
                        if RYU_Y[entry_id] != OPPONENT_Y[entry_id]{ // If Ryu's Y and the Opponent's Y aren't equal, do the following:
                            StatusModule::set_situation_kind(boma, smash::app::SituationKind(*SITUATION_KIND_AIR), true); // Set Ryu to be airborne
                            VERT_EXTRA[entry_id] = 12.0; // Set the extra vertical distance
                        }
                        else {
                            VERT_EXTRA[entry_id] = 0.0; // If both Ryu and his opponent are on the same Y, he just slides instead
                        }
                        PostureModule::set_pos_2d(module_accessor, &Vector2f{ // Linear Interpolation formula: Destination * t + Starting * (1.0 - t), where 0 <= t <= 1. You can't add vectors apparently, so I did this for both X and Y.
                            x: (((OPPONENT_POS[entry_id].x + OPPONENT_DIRECTION[entry_id]) * SEC_SEN_TIMER[entry_id]) + RYU_POS[entry_id].x * (1.0 - SEC_SEN_TIMER[entry_id])),
                            y: (((OPPONENT_POS[entry_id].y + 12.0) * SEC_SEN_TIMER[entry_id]) + RYU_POS[entry_id].y * (1.0 - SEC_SEN_TIMER[entry_id])) // There's a +12.0 so that, for moving into the air, Ryu moves slightly above the opponent. Does nothing on the ground. I may change this later.
                        });
                    }
                    SEC_SEN_TIMER[entry_id] += 0.1; // Increases the "t" in the interpolation formula by 0.1 every frame.
                    if SEC_SEN_TIMER[entry_id] > 1.0 {
                        WorkModule::on_flag(module_accessor, *FIGHTER_INSTANCE_WORK_ID_FLAG_NO_SPEED_OPERATION_CHK); // These three lines are here to make sure Ryu doesn't just fall like a rock after movinag into the air.
                        acmd!(lua_state,{
                            SET_SPEED_EX(0, 0.5, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN)
                            CAM_ZOOM_OUT() // Resets the camera.
                        });
                        WorkModule::off_flag(module_accessor, *FIGHTER_INSTANCE_WORK_ID_FLAG_NO_SPEED_OPERATION_CHK);
                        if StatusModule::situation_kind(module_accessor) == *SITUATION_KIND_AIR {
                            PostureModule::reverse_lr(module_accessor); // meant to turn Ryu around if he's in the air, since he moves behind the opponent, though I just moved it here, still untested if it works
                        }
                        SlowModule::clear_whole(module_accessor); // Clears the global 4x slowdown multiplier from above
                        JostleModule::set_status(module_accessor, true); // Resets Ryu's body blocking back to normal
                        SEC_SEN_TIMER[entry_id] = -0.4; // Resets the interpolation timer.
                        SECRET_SENSATION[entry_id] = false;
                        CAMERA[entry_id] = false;
                    }
                }
            }
        }
    }
}

pub fn install() {
    acmd::add_custom_hooks!(once_per_fighter_frame);
}