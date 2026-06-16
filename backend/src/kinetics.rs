use crate::models::{WaterQuality, ProcessConfig};

pub struct KineticsParams {
    pub mu_max_nitrif: f64,
    pub ks_nh3: f64,
    pub theta_temp: f64,
    pub k_bod_oxidation: f64,
    pub denitrification_rate: f64,
    pub cn_ratio_required: f64,
    pub p_release_rate: f64,
    pub p_uptake_factor: f64,
}

impl Default for KineticsParams {
    fn default() -> Self {
        Self {
            mu_max_nitrif: 0.8,
            ks_nh3: 1.0,
            theta_temp: 1.07,
            k_bod_oxidation: 0.25,
            denitrification_rate: 0.03,
            cn_ratio_required: 4.0,
            p_release_rate: 0.02,
            p_uptake_factor: 1.75,
        }
    }
}

pub fn temperature_correction(temp: f64, theta: f64) -> f64 {
    theta.powf(temp - 20.0)
}

pub fn temp_correction_factor_low(temp: f64) -> f64 {
    if temp >= 15.0 {
        1.0
    } else {
        (0.94_f64).powf(15.0 - temp)
    }
}

pub fn monod_equation(substrate: f64, ks: f64) -> f64 {
    if substrate <= 0.0 {
        return 0.0;
    }
    substrate / (ks + substrate)
}

pub fn calculate_aerobic_reactions(
    input: &WaterQuality,
    config: &ProcessConfig,
    temp: f64,
    dt_days: f64,
    kinetics: &KineticsParams,
) -> WaterQuality {
    let mut output = input.clone();
    let temp_corr = temperature_correction(temp, kinetics.theta_temp);
    let temp_low_corr = temp_correction_factor_low(temp);
    let overall_corr = temp_corr * temp_low_corr;

    let mlss_vss = config.aerobic.mlss * 0.7;
    let biomass = mlss_vss / 1000.0;

    let nitrification_rate = kinetics.mu_max_nitrif
        * monod_equation(output.nh3_n, kinetics.ks_nh3)
        * monod_equation(output.do, 0.5)
        * overall_corr;

    let nh3_removed = (nitrification_rate * biomass * output.nh3_n * dt_days).min(output.nh3_n);
    output.nh3_n -= nh3_removed;
    output.no3_n += nh3_removed;

    let bod_removal_rate = kinetics.k_bod_oxidation
        * monod_equation(output.do, 0.2)
        * overall_corr;
    let bod_removed = (bod_removal_rate * output.bod5 * dt_days).min(output.bod5);
    output.bod5 -= bod_removed;
    output.cod -= bod_removed * 1.5;

    let cod_removal_rate = kinetics.k_bod_oxidation * 0.5 * overall_corr;
    let cod_removed = (cod_removal_rate * output.cod * dt_days).min(output.cod);
    output.cod -= cod_removed;

    let p_uptake_rate = kinetics.p_release_rate * kinetics.p_uptake_factor
        * monod_equation(output.do, 0.3)
        * overall_corr;
    let po4_uptaken = (p_uptake_rate * biomass * dt_days * 1000.0).min(output.po4_p);
    output.po4_p -= po4_uptaken;
    output.tp -= po4_uptaken;

    output.vfa *= (0.95_f64).powf(dt_days * 24.0);

    let do_consumption = (bod_removed * 0.5 + nh3_removed * 4.57) * dt_days * 24.0;
    let do_supply = config.aeration_rate / config.aerobic.volume * 0.25;
    output.do = (output.do - do_consumption + do_supply * dt_days).clamp(0.0, config.aerobic.do_setpoint * 1.2);
    output.do = output.do.max(0.1);

    output.tn = output.nh3_n + output.no3_n + (input.tn - input.nh3_n - input.no3_n) * 0.5;

    output
}

pub fn calculate_anoxic_reactions(
    input: &WaterQuality,
    return_sludge: &WaterQuality,
    internal_recirc: &WaterQuality,
    config: &ProcessConfig,
    temp: f64,
    dt_days: f64,
    kinetics: &KineticsParams,
) -> WaterQuality {
    let r = config.sludge_recirculation_ratio;
    let ri = config.internal_recirculation_ratio;
    let total_flow = 1.0 + r + ri;

    let mut mixed = WaterQuality::empty();
    mixed.cod = (input.cod + r * return_sludge.cod + ri * internal_recirc.cod) / total_flow;
    mixed.bod5 = (input.bod5 + r * return_sludge.bod5 + ri * internal_recirc.bod5) / total_flow;
    mixed.ss = (input.ss + r * return_sludge.ss + ri * internal_recirc.ss) / total_flow;
    mixed.nh3_n = (input.nh3_n + r * return_sludge.nh3_n + ri * internal_recirc.nh3_n) / total_flow;
    mixed.no3_n = (input.no3_n + r * return_sludge.no3_n + ri * internal_recirc.no3_n) / total_flow;
    mixed.tp = (input.tp + r * return_sludge.tp + ri * internal_recirc.tp) / total_flow;
    mixed.po4_p = (input.po4_p + r * return_sludge.po4_p + ri * internal_recirc.po4_p) / total_flow;
    mixed.vfa = (input.vfa + r * return_sludge.vfa + ri * internal_recirc.vfa) / total_flow;
    mixed.do = (input.do + r * return_sludge.do + ri * internal_recirc.do) / total_flow;
    mixed.tn = (input.tn + r * return_sludge.tn + ri * internal_recirc.tn) / total_flow;

    let mut output = mixed.clone();
    let temp_corr = temperature_correction(temp, kinetics.theta_temp);
    let temp_low_corr = temp_correction_factor_low(temp);
    let overall_corr = temp_corr * temp_low_corr;

    let mlss_vss = config.anoxic.mlss * 0.7;
    let biomass = mlss_vss / 1000.0;

    let cn_available = mixed.vfa / (mixed.no3_n + 0.001);
    let cn_factor = if cn_available >= kinetics.cn_ratio_required {
        1.0
    } else {
        cn_available / kinetics.cn_ratio_required
    };

    let denit_rate = kinetics.denitrification_rate
        * monod_equation(mixed.no3_n, 0.5)
        * (1.0 - monod_equation(mixed.do, 0.5))
        * cn_factor
        * overall_corr;

    let no3_removed = (denit_rate * biomass * mixed.no3_n * dt_days * 1000.0).min(mixed.no3_n);
    output.no3_n -= no3_removed;

    let bod_used = no3_removed * kinetics.cn_ratio_required * 1.5;
    output.bod5 = (output.bod5 - bod_used).max(0.0);
    output.vfa = (output.vfa - no3_removed * kinetics.cn_ratio_required).max(0.0);
    output.cod = (output.cod - bod_used).max(0.0);

    output.tn = output.nh3_n + output.no3_n + (mixed.tn - mixed.nh3_n - mixed.no3_n) * 0.7;

    output.do = (output.do * 0.3).max(0.05);

    output
}

pub fn calculate_anaerobic_reactions(
    input: &WaterQuality,
    return_sludge: &WaterQuality,
    config: &ProcessConfig,
    temp: f64,
    dt_days: f64,
    kinetics: &KineticsParams,
) -> WaterQuality {
    let r = config.sludge_recirculation_ratio;
    let total_flow = 1.0 + r;

    let mut mixed = WaterQuality::empty();
    mixed.cod = (input.cod + r * return_sludge.cod) / total_flow;
    mixed.bod5 = (input.bod5 + r * return_sludge.bod5) / total_flow;
    mixed.ss = (input.ss + r * return_sludge.ss) / total_flow;
    mixed.nh3_n = (input.nh3_n + r * return_sludge.nh3_n) / total_flow;
    mixed.no3_n = (input.no3_n + r * return_sludge.no3_n) / total_flow;
    mixed.tp = (input.tp + r * return_sludge.tp) / total_flow;
    mixed.po4_p = (input.po4_p + r * return_sludge.po4_p) / total_flow;
    mixed.vfa = (input.vfa + r * return_sludge.vfa) / total_flow;
    mixed.do = (input.do + r * return_sludge.do) / total_flow;
    mixed.tn = (input.tn + r * return_sludge.tn) / total_flow;

    let mut output = mixed.clone();
    let temp_corr = temperature_correction(temp, kinetics.theta_temp);
    let temp_low_corr = temp_correction_factor_low(temp);
    let overall_corr = temp_corr * temp_low_corr;

    let mlss_vss = config.anaerobic.mlss * 0.7;
    let biomass = mlss_vss / 1000.0;

    let p_release_rate = kinetics.p_release_rate
        * monod_equation(mixed.vfa, 10.0)
        * (1.0 - monod_equation(mixed.do, 0.2))
        * (1.0 - monod_equation(mixed.no3_n, 1.0))
        * overall_corr;

    let p_released = p_release_rate * biomass * dt_days * 1000.0;
    output.po4_p += p_released;
    output.tp += p_released * 0.5;

    output.vfa *= (0.98_f64).powf(dt_days * 24.0);

    let fermentation_bod = mixed.bod5 * 0.1 * dt_days;
    output.bod5 = (output.bod5 - fermentation_bod).max(0.0);
    output.vfa += fermentation_bod * 0.8;

    output.do = (output.do * 0.05).max(0.01);
    output.no3_n *= 0.95;

    output
}

pub fn calculate_clarifier(
    input: &WaterQuality,
    config: &ProcessConfig,
) -> (WaterQuality, WaterQuality) {
    let mut effluent = input.clone();
    let mut return_sludge = input.clone();

    let ss_removal = 0.95;
    let ss_removed = effluent.ss * ss_removal;
    effluent.ss -= ss_removed;

    let particulate_p = effluent.tp * 0.4;
    let p_removed_with_ss = particulate_p * ss_removal;
    effluent.tp -= p_removed_with_ss;

    let particulate_cod = effluent.cod * 0.3;
    let cod_removed_with_ss = particulate_cod * ss_removal;
    effluent.cod -= cod_removed_with_ss;

    effluent.bod5 = effluent.bod5 * 0.98;

    let r = config.sludge_recirculation_ratio;
    return_sludge.ss = input.ss * (1.0 + r) / r * 0.95;
    return_sludge.tp = input.tp * 1.1;
    return_sludge.cod = input.cod * 1.2;

    (effluent, return_sludge)
}
