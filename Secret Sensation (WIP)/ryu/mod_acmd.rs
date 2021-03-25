use smash::lib::lua_const::*;
use smash::app::lua_bind::*;
use smash::lua2cpp::L2CFighterCommon;
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
static mut SEC_SEN_STATE : [bool; 8] = [false; 8];
static mut SEC_SEN_DIREC : [i32; 8] = [0; 8];

pub fn once_per_fighter_frame(fighter : &mut L2CFighterCommon) {
    unsafe {
        let module_accessor = smash::app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
        let entry_id = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
        let fighter_kind = smash::app::utility::get_kind(module_accessor);
        let lua_state = fighter.lua_state_agent; 
       
        
        if fighter_kind == *FIGHTER_KIND_RYU {
            if entry_id < 8 {
                
                if (MotionModule::motion_kind(module_accessor) == smash::hash40("appeal_hi_r") // Sets the damage Ryu takes to (effectively) 0 during the first 30 frames of up taunt.
                || MotionModule::motion_kind(module_accessor) == smash::hash40("appeal_hi_l"))
                && MotionModule::frame(module_accessor) <= 30.0 {
                    SEC_SEN_STATE[entry_id] = true;
                }
                else if SECRET_SENSATION[entry_id] {
                    acmd!(lua_state,{
                        WHOLE_HIT(HIT_STATUS_XLU)
                    });
                    DamageModule::set_damage_mul(module_accessor, 0.0);
                    DamageModule::set_reaction_mul(module_accessor, 0.0);
                }
                else if SECRET_SENSATION[entry_id] == false
                && SEC_SEN_STATE[entry_id] {
                    DamageModule::set_damage_mul(module_accessor, 1.0);
                    DamageModule::set_reaction_mul(module_accessor, 1.0);
                    SEC_SEN_STATE[entry_id] = false;
                    acmd!(lua_state,{
                        WHOLE_HIT(HIT_STATUS_NORMAL)
                    });
                }

                if SEC_SEN_STATE[entry_id] {
                    DamageModule::set_damage_mul(module_accessor, 0.0);
                    DamageModule::set_reaction_mul(module_accessor, 0.0);
                }

                // Reset Vars

                if StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_REBIRTH || smash::app::sv_information::is_ready_go() == false {
                    SECRET_SENSATION[entry_id] = false;
                    SEC_SEN_STATE[entry_id] = false;
                }
                
                // Secret Sensation???

                if SECRET_SENSATION[entry_id] {
                    JostleModule::set_status(module_accessor, false); // Turns off body blocking for Ryu every frame Secret Sensation is true
                    if CAMERA[entry_id] == false { // Exists so all of this code will only happen once.
                        acmd!(lua_state,{
                            CAM_ZOOM_IN_arg5(5.0, 0.0, 1.5, 0.0, 0.0); // Sets the camera
                            SLOW_OPPONENT(100.0, 32.0) // Slows the opponent down by 100x for 32 frames
                            FILL_SCREEN_MODEL_COLOR(0, 3, 0.2, 0.2, 0.2, 0, 0, 0, 1, 1, *smash::lib::lua_const::EffectScreenLayer::GROUND, 205); // Darkens everything on screen except for the Fighters
                        });
                        SlowModule::set_whole(module_accessor, 4, 0); // Slows ***everything*** down by a 4x. This includes the above slowdown, which probably means I should shorten the above length of time but eh
                        RYU_X[entry_id] = PostureModule::pos_x(module_accessor); // Gets Ryu's position
                        RYU_Y[entry_id] = PostureModule::pos_y(module_accessor);
                        if RYU_X[entry_id] == OPPONENT_X[entry_id] { // The reason I sometimes set Ryu's position as the Opponent's position is for this, if Ryu can't find the owner of what hit him, he will instead just dodge backwards.
                            OPPONENT_DIRECTION[entry_id] = -12.0 * PostureModule::lr(module_accessor);
                            SEC_SEN_DIREC[entry_id] = *FIGHTER_RYU_STATUS_KIND_SPECIAL_LW_STEP_B;
                        }
                        else if RYU_X[entry_id] < OPPONENT_X[entry_id] { // Checks where Ryu and his Opponent are relative to each other, and sets a value so Ryu always moves *behind* the opponent
                            OPPONENT_DIRECTION[entry_id] = 12.0;
                            if PostureModule::lr(module_accessor) == -1.0 {
                                PostureModule::reverse_lr(module_accessor);
                            }
                            SEC_SEN_DIREC[entry_id] = *FIGHTER_RYU_STATUS_KIND_SPECIAL_LW_STEP_F;
                        }
                        else {
                            OPPONENT_DIRECTION[entry_id] = -12.0;
                            if PostureModule::lr(module_accessor) == 1.0 {
                                PostureModule::reverse_lr(module_accessor);
                            }
                            SEC_SEN_DIREC[entry_id] = *FIGHTER_RYU_STATUS_KIND_SPECIAL_LW_STEP_F;
                        }
                        CAMERA[entry_id] = true; // Again, ensures that the above code only runs once.
                    }
                    if StatusModule::status_kind(module_accessor) != SEC_SEN_DIREC[entry_id] { // Checks every frame if Ryu is in Focus Attack Dash state. Use another status, perhaps ESCAPE, if you're using another character
                        KineticModule::change_kinetic(module_accessor, *FIGHTER_KINETIC_TYPE_RESET);
                        StatusModule::change_status_request_from_script(module_accessor, SEC_SEN_DIREC[entry_id], true);
                    }
                    if SEC_SEN_TIMER[entry_id] >= 0.0 { // This whole if statement is for linearly interpolating Ryu's position, instead of just teleporting him behind the opponent.
                        if RYU_Y[entry_id] != OPPONENT_Y[entry_id]{ // If Ryu's Y and the Opponent's Y aren't equal, do the following:
                            StatusModule::set_situation_kind(module_accessor, smash::app::SituationKind(*SITUATION_KIND_AIR), true); // Set Ryu to be airborne
                            VERT_EXTRA[entry_id] = 12.0; // Set the extra vertical distance
                        }
                        else {
                            VERT_EXTRA[entry_id] = 0.0; // If both Ryu and his opponent are on the same Y, he just slides instead
                        }
                        PostureModule::set_pos_2d(module_accessor, &Vector2f{ // Linear Interpolation formula: Destination * t + Starting * (1.0 - t), where 0 <= t <= 1. You can't add vectors apparently, so I did this for both X and Y.
                            x: (((OPPONENT_X[entry_id] + OPPONENT_DIRECTION[entry_id]) * SEC_SEN_TIMER[entry_id]) + RYU_X[entry_id] * (1.0 - SEC_SEN_TIMER[entry_id])),
                            y: (((OPPONENT_Y[entry_id] + VERT_EXTRA[entry_id]) * SEC_SEN_TIMER[entry_id]) + RYU_Y[entry_id] * (1.0 - SEC_SEN_TIMER[entry_id])) // There's a +12.0 so that, for moving into the air, Ryu moves slightly above the opponent. Does nothing on the ground. I may change this later.
                        });
                    }
                    SEC_SEN_TIMER[entry_id] += 0.1; // Increases the "t" in the interpolation formula by 0.1 every frame.
                    if SEC_SEN_TIMER[entry_id] > 1.0 {
                        SECRET_SENSATION[entry_id] = false;
                        CAMERA[entry_id] = false;
                        WorkModule::on_flag(module_accessor, *FIGHTER_INSTANCE_WORK_ID_FLAG_NO_SPEED_OPERATION_CHK);
                        acmd!(lua_state,{
                            SET_SPEED_EX(0, 0.5, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN) // Here to make sure Ryu doesn't just fall like a rock after movinag into the air.
                            CAM_ZOOM_OUT() // Resets the camera.
                            CANCEL_FILL_SCREEN(0, 5) // Clears out the background screen darken effect.
                        });
                        WorkModule::off_flag(module_accessor, *FIGHTER_INSTANCE_WORK_ID_FLAG_NO_SPEED_OPERATION_CHK);
                        if StatusModule::situation_kind(module_accessor) == *SITUATION_KIND_AIR {
                            StatusModule::change_status_request_from_script(module_accessor, *FIGHTER_RYU_STATUS_KIND_TURN_AUTO, true); // Forces him into his auto-turn state, so he properly turns around in the air.
                            // PostureModule::reverse_lr(module_accessor); // If you're porting this to other fighters, use this instead.
                        }
                        SlowModule::clear_whole(module_accessor); // Clears the global 4x slowdown multiplier from above
                        JostleModule::set_status(module_accessor, true); // Resets Ryu's body blocking back to normal
                        SEC_SEN_TIMER[entry_id] = -0.4; // Resets the interpolation timer.
                    }
                }
            }
        }
    }
}

pub fn install() {
    acmd::add_custom_hooks!(once_per_fighter_frame);
}