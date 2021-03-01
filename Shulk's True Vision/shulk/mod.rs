use smash::phx::Hash40;
use smash::lua2cpp::L2CAgentBase;
use smash::app::sv_animcmd;
use smash::lib::lua_const::*;
use smash::app::lua_bind::*;
use smash_script::*;
use smash_script::macros;
use smash::phx::Vector3f;
use smash::app::BattleObjectModuleAccessor;
use smash::app::lua_bind::EffectModule;

pub static mut TIME_SLOW_EFFECT_VECTOR: smash::phx::Vector3f = smash::phx::Vector3f {x:-3.0,y:3.0,z:0.0};

pub unsafe fn entry_id(module_accessor: &mut BattleObjectModuleAccessor) -> usize {
    let entry_id = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    return entry_id;
}

static mut SPECIAL_LW : [bool; 8] = [false; 8];
static mut SPECIAL_LW_TIMER : [i16; 8] = [-1; 8];
static mut DAMAGE_TAKEN : [f32; 8] = [0.0; 8];

#[skyline::hook(replace=smash::app::lua_bind::WorkModule::is_enable_transition_term)]
pub unsafe fn shulk_is_enable_transition_term_replace(module_accessor: &mut BattleObjectModuleAccessor, term: i32) -> bool {
    let fighter_kind = smash::app::utility::get_kind(module_accessor);
    let ret = original!()(module_accessor,term);
    if fighter_kind == *FIGHTER_KIND_SHULK {
        if SPECIAL_LW[entry_id(module_accessor)] {
            if term == *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_SPECIAL_LW {
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
    else {
        return ret;
    }
}

#[fighter_frame( agent = FIGHTER_KIND_SHULK )]
unsafe fn shulk_frame(fighter: &mut L2CAgentBase) {
    let boma = smash::app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;

    if entry_id < 8 {
        
        // Reset Vars

        if StatusModule::status_kind(boma) == *FIGHTER_STATUS_KIND_REBIRTH || smash::app::sv_information::is_ready_go() == false {
            SPECIAL_LW[entry_id] = false;
            SPECIAL_LW_TIMER[entry_id] = -1;
        }

        // Damage Check

        if StopModule::is_damage(boma) {
            DAMAGE_TAKEN[entry_id] = WorkModule::get_float(boma,*FIGHTER_INSTANCE_WORK_ID_FLOAT_SUCCEED_HIT_DAMAGE);
            if ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_GUARD) {
                if SPECIAL_LW[entry_id] == false {
                    let launch_speed = KineticModule::get_sum_speed_x(boma,*KINETIC_ENERGY_RESERVE_ATTRIBUTE_DAMAGE);
                    let facing_dirn = PostureModule::lr(boma);
                    if launch_speed > 0.0 && facing_dirn > 0.0 {
                        PostureModule::reverse_lr(boma);
                    }
                    else if launch_speed < 0.0 && facing_dirn < 0.0 {
                        PostureModule::reverse_lr(boma);
                    }
                    DamageModule::add_damage(boma, DAMAGE_TAKEN[entry_id] * -0.5, 0);
                    KineticModule::change_kinetic(boma,*FIGHTER_KINETIC_TYPE_RESET);
                    StatusModule::change_status_request_from_script(boma, *FIGHTER_SHULK_STATUS_KIND_SPECIAL_LW_HIT, true);
                    WorkModule::set_float(boma, 0.0, *FIGHTER_SHULK_INSTANCE_WORK_ID_FLOAT_SPECIAL_LW_ATTACK_POWER);
                    WorkModule::set_int(boma, *FIGHTER_SHULK_MONAD_TYPE_DEFAULT, *FIGHTER_SHULK_INSTANCE_WORK_ID_INT_SPECIAL_N_TYPE);
                    WorkModule::set_int(boma, *FIGHTER_SHULK_MONAD_TYPE_DEFAULT, *FIGHTER_SHULK_INSTANCE_WORK_ID_INT_SPECIAL_N_TYPE_SELECT);
                    SPECIAL_LW[entry_id] = true;
                    SPECIAL_LW_TIMER[entry_id] = 3600;
                }
            }
        }

        // Special Lw Check

        if StatusModule::status_kind(boma) == *FIGHTER_SHULK_STATUS_KIND_SPECIAL_LW_HIT {
            EffectModule::req_on_joint(boma, Hash40::new("sys_sp_flash"), Hash40::new("head"), &TIME_SLOW_EFFECT_VECTOR, &TIME_SLOW_EFFECT_VECTOR, 1.0, &TIME_SLOW_EFFECT_VECTOR, &TIME_SLOW_EFFECT_VECTOR, false, 0, 0, 0);
        }
        if SPECIAL_LW_TIMER[entry_id] > 0 {
            SPECIAL_LW_TIMER[entry_id] = SPECIAL_LW_TIMER[entry_id] - 1;
        }
        else if SPECIAL_LW_TIMER[entry_id] == 0 {
            SPECIAL_LW_TIMER[entry_id] = -1;
            let pos: Vector3f = Vector3f{x: 0.0, y: 13.0, z: 0.0};
            let rot: Vector3f = Vector3f{x: 0.0, y: 90.0, z: 0.0};
            let countereff: u32 = EffectModule::req_follow(boma, Hash40::new("sys_counter_flash"), Hash40::new("top"), &pos, &rot, 1.0, false, 0, 0, 0, 0, 0, false, false) as u32;
            EffectModule::set_rgb(boma, countereff, 0.0, 5.0, 5.0);
        }
        else{
            SPECIAL_LW[entry_id] = false;
        }
    }
}

#[script( agent = "shulk", scripts = ["game_speciallwattack", "game_specialairlwattack"], category = ACMD_GAME )]
unsafe fn shulk_counterattack(fighter: &mut L2CAgentBase) {
    let lua_state = fighter.lua_state_agent;
    let boma = smash::app::sv_system::battle_object_module_accessor(lua_state);
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    sv_animcmd::frame(lua_state, 8.0);
    let dmg: f32;
    if SPECIAL_LW[entry_id] == false {
        dmg = 10.0;
    }
    else {
        dmg = 8.0;
    }
    if macros::is_excute(fighter) {
        smash_script::macros::ATTACK(fighter, 0, 0, Hash40::new("top"), dmg, 50, 80, 0, 70, 12.0, 0.0, 10.5, 28.0, Some(0.0), Some(10.5), Some(20.5), 2.0, 1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_SWORD);
        smash_script::macros::ATTACK(fighter, 1, 0, Hash40::new("top"), dmg - 1.0, 50, 80, 0, 70, 7.0, 0.0, 10.5, 33.0, Some(0.0), Some(10.5), Some(9.5), 2.0, 1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_SWORD);
        AttackModule::set_force_reaction(boma, 0, true, false);
        AttackModule::set_force_reaction(boma, 1, true, false);
    }
    sv_animcmd::wait(lua_state, 2.0);
    if macros::is_excute(fighter) {
        smash_script::macros::ATTACK(fighter, 0, 0, Hash40::new("top"), dmg, 50, 80, 0, 70, 12.0, 0.0, 10.5, 25.0, Some(0.0), Some(10.5), Some(17.5), 2.0, 1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_SWORD);
        smash_script::macros::ATTACK(fighter, 1, 0, Hash40::new("top"), dmg -1.0, 50, 80, 0, 70, 7.0, 0.0, 10.5, 30.0, Some(0.0), Some(10.5), Some(6.5), 2.0, 1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_F, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_SWORD);
        AttackModule::set_force_reaction(boma, 0, true, false);
        AttackModule::set_force_reaction(boma, 1, true, false);
    }
    sv_animcmd::wait(lua_state, 1.0);
    if macros::is_excute(fighter) {
        AttackModule::clear_all(boma);
    }
    macros::FT_MOTION_RATE(fighter, 0.8);
}

#[script( agent = "shulk", script = "game_speciallwf", category = ACMD_GAME )]
unsafe fn shulk_counterforward(fighter: &mut L2CAgentBase) {
    let lua_state = fighter.lua_state_agent;
    let boma = smash::app::sv_system::battle_object_module_accessor(lua_state);
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    macros::FT_MOTION_RATE(fighter, 0.8);
    sv_animcmd::frame(lua_state, 20.0);
    let dmg: f32;
    if SPECIAL_LW[entry_id] == false {
        dmg = 13.0;
    }
    else {
        dmg = 8.0;
    }
    if macros::is_excute(fighter) {
        smash_script::macros::ATTACK(fighter, 0, 0, Hash40::new("top"), dmg, 50, 90, 0, 70, 11.0, 0.0, 9.0, -24.5, Some(0.0), Some(9.0), Some(-15.0), 2.0, 1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_B, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_SWORD);
        smash_script::macros::ATTACK(fighter, 1, 0, Hash40::new("top"), dmg - 1.0, 50, 90, 0, 70, 7.0, 0.0, 9.0, -28.5, Some(0.0), Some(9.0), Some(-3.0), 2.0, 1.0, *ATTACK_SETOFF_KIND_OFF, *ATTACK_LR_CHECK_B, false, 0, 0.0, 0, false, false, false, false, true, *COLLISION_SITUATION_MASK_GA, *COLLISION_CATEGORY_MASK_ALL, *COLLISION_PART_MASK_ALL, false, Hash40::new("collision_attr_cutup"), *ATTACK_SOUND_LEVEL_L, *COLLISION_SOUND_ATTR_CUTUP, *ATTACK_REGION_SWORD);
        AttackModule::set_force_reaction(boma, 0, true, false);
        AttackModule::set_force_reaction(boma, 1, true, false);
    }
    sv_animcmd::wait(lua_state, 2.0);
    if macros::is_excute(fighter) {
        AttackModule::clear_all(boma);
    }
}

pub fn install() {
    smash_script::replace_fighter_frames!(
        shulk_frame
    );
    smash_script::replace_scripts!(
        shulk_counterattack,
        shulk_counterforward
    );
    skyline::install_hook!(shulk_is_enable_transition_term_replace);
}
