use smash::phx::Hash40;
use smash::lua2cpp::L2CAgentBase;
use smash::lib::lua_const::*;
use smash::app::lua_bind::*;
use smash_script::*;
use smash::phx::Vector2f;

pub static mut SECRET_SENSATION : [bool; 8] = [false; 8];
pub static mut CAMERA : [bool; 8] = [false; 8];
pub static mut OPPONENT_X : [f32; 8] = [0.0; 8];
pub static mut OPPONENT_Y : [f32; 8] = [0.0; 8];
static mut RYU_X : [f32; 8] = [0.0; 8];
static mut RYU_Y : [f32; 8] = [0.0; 8];
static mut SEC_SEN_TIMER : [f32; 8] = [-0.4; 8]; // I start this as -0.4 so that Ryu doesn't immediately start dodging, there's a little pause before he does
static mut OPPONENT_DIRECTION : [f32; 8] = [12.0; 8];
static mut VERT_EXTRA : [f32; 8] = [12.0; 8];

#[fighter_frame( agent = FIGHTER_KIND_RYU )]
unsafe fn ryu_frame(fighter: &mut L2CAgentBase) {
    let boma = smash::app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    if entry_id < 8 {

        if (MotionModule::motion_kind(boma) == smash::hash40("appeal_hi_r") // Sets the damage Ryu takes to (effectively) 0 during the first 30 frames of up taunt.
        || MotionModule::motion_kind(boma) == smash::hash40("appeal_hi_l"))
        && MotionModule::frame(boma) <= 30.0 {
            DamageModule::set_damage_mul(boma, 0.000001);
        }
        else { // Resets his damage multiplier back to 1.0. If there's anything that also changes how much damage Ryu takes, however, then it will break that atm, but it's fine for items off at the moment.
            DamageModule::set_damage_mul(boma, 1.0);
        }

        // Secret Sensation???

        if SECRET_SENSATION[entry_id] {
            if CAMERA[entry_id] == false { // Exists so all of this code will only happen once.
                KineticModule::change_kinetic(boma, *FIGHTER_KINETIC_TYPE_RESET); // Resets all knockback
                StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_WAIT, true); // Sets Ryu into the idle state
                macros::CAM_ZOOM_IN_arg5(fighter, 5.0, 0.0, 1.1, 0.0, 0.0); // Sets the camera
                macros::SLOW_OPPONENT(fighter, 100.0, 32.0); // Slows the opponent down by 100x for 32 frames
                SlowModule::set_whole(boma, 4, 0); // Slows ***everything*** down by a 4x. This includes the above slowdown, which probably means I should shorten the above length of time but eh
                JostleModule::set_status(boma, false); // It *should* turn off body blocking for Ryu but I'm not sure, I couldn't tell
                MotionModule::change_motion_inherit_frame_keep_rate(boma,Hash40::new("turn"),0.0,0.0,0.0); // doesn't actually work, but is supposed to change his animation to the turn animation when switching directions
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
                if RYU_Y[entry_id] != OPPONENT_Y[entry_id]{
                    StatusModule::set_situation_kind(boma, smash::app::SituationKind(*SITUATION_KIND_AIR), true);
                    VERT_EXTRA[entry_id] = 12.0;
                }
                PostureModule::set_pos_2d(boma, &Vector2f{ // Linear Interpolation formula: Destination * t + Starting * (1.0 - t), where 0 <= t <= 1. You can't add vectors apparently, so I did this for both X and Y.
                    x: (((OPPONENT_X[entry_id] + OPPONENT_DIRECTION[entry_id]) * SEC_SEN_TIMER[entry_id]) + RYU_X[entry_id] * (1.0 - SEC_SEN_TIMER[entry_id])),
                    y: (((OPPONENT_Y[entry_id] + VERT_EXTRA[entry_id]) * SEC_SEN_TIMER[entry_id]) + RYU_Y[entry_id] * (1.0 - SEC_SEN_TIMER[entry_id])) // There's a +12.0 so that, for moving into the air, Ryu moves slightly above the opponent. Does nothing on the ground. I may change this later.
                });
            }
            SEC_SEN_TIMER[entry_id] += 0.1; // Increases the "t" in the interpolation formula by 0.1 every frame.
            if SEC_SEN_TIMER[entry_id] > 1.0 {
                WorkModule::on_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_NO_SPEED_OPERATION_CHK); // These three lines are here to make sure Ryu doesn't just fall like a rock after moving into the air.
                macros::SET_SPEED_EX(fighter, 0, 0.5, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
                WorkModule::off_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_NO_SPEED_OPERATION_CHK);
                if StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR {
                    PostureModule::reverse_lr(boma); // apparently it works
                }
                macros::CAM_ZOOM_OUT(fighter); // Resets the camera. You'll have to add this to your macros.rs youself!
                SlowModule::clear_whole(boma); // Clears the global 4x slowdown multiplier from above
                JostleModule::set_status(boma, true); // Resets Ryu's body blocking back to normal
                SEC_SEN_TIMER[entry_id] = -0.4; // Resets the interpolation timer.
                SECRET_SENSATION[entry_id] = false;
                CAMERA[entry_id] = false;
            }
        }
    }
}

pub fn install() {
    smash_script::replace_fighter_frames!(
        ryu_frame
    );
}