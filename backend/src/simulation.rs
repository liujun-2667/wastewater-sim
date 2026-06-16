use crate::models::*;
use crate::kinetics::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStepResult {
    pub time_hours: f64,
    pub anaerobic_effluent: WaterQuality,
    pub anoxic_effluent: WaterQuality,
    pub aerobic_effluent: WaterQuality,
    pub final_effluent: WaterQuality,
    pub sludge_production: f64,
    pub oxygen_demand: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub success: bool,
    pub message: String,
    pub influent_warnings: Vec<String>,
    pub time_series: Vec<TimeStepResult>,
    pub final_effluent: WaterQuality,
    pub unit_effluents: UnitEffluents,
    pub compliance: ComplianceResult,
    pub mass_balance: MassBalanceResult,
    pub energy_consumption: EnergyResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitEffluents {
    pub anaerobic: WaterQuality,
    pub anoxic: WaterQuality,
    pub aerobic: WaterQuality,
    pub clarifier: WaterQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub cod: (f64, f64, bool),
    pub bod5: (f64, f64, bool),
    pub ss: (f64, f64, bool),
    pub tn: (f64, f64, bool),
    pub nh3_n: (f64, f64, bool),
    pub tp: (f64, f64, bool),
    pub overall_compliant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassBalanceResult {
    pub nitrogen_flows: Vec<(String, String, f64)>,
    pub phosphorus_flows: Vec<(String, String, f64)>,
    pub carbon_flows: Vec<(String, String, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyResult {
    pub aeration_kwh: f64,
    pub mixing_kwh: f64,
    pub pumping_kwh: f64,
    pub total_kwh: f64,
    pub kwh_per_m3: f64,
    pub breakdown: Vec<(String, f64, f64)>,
}

pub fn run_simulation(
    influent: &InfluentParams,
    process: &ProcessConfig,
    sim_config: &SimulationConfig,
) -> SimulationResult {
    let warnings = influent.validate();
    let kinetics = KineticsParams::default();

    let dt_days = sim_config.time_step_hours / 24.0;
    let total_steps = (sim_config.total_duration_days * 24.0 / sim_config.time_step_hours) as usize;

    let mut time_series: Vec<TimeStepResult> = Vec::new();

    let inflow_wq = WaterQuality::from_influent(influent);
    let mut anaerobic_eff = inflow_wq.clone();
    let mut anoxic_eff = inflow_wq.clone();
    let mut aerobic_eff = inflow_wq.clone();
    let mut final_eff = inflow_wq.clone();
    let mut return_sludge = inflow_wq.clone();
    let mut internal_recirc = inflow_wq.clone();

    let mut success = true;
    let mut error_msg = String::from("仿真成功完成");

    for step in 0..total_steps {
        let time_hours = step as f64 * sim_config.time_step_hours;

        let new_anaerobic = calculate_anaerobic_reactions(
            &inflow_wq, &return_sludge, process, influent.temperature, dt_days, &kinetics
        );

        if let Err(e) = new_anaerobic.check_valid() {
            success = false;
            error_msg = format!("厌氧池计算异常: {}. 可能原因: 碳源不足或NO3-N积累过高", e);
            break;
        }

        let new_anoxic = calculate_anoxic_reactions(
            &new_anaerobic, &return_sludge, &internal_recirc, process, influent.temperature, dt_days, &kinetics
        );

        if let Err(e) = new_anoxic.check_valid() {
            success = false;
            error_msg = format!("缺氧池计算异常: {}. 可能原因: 碳源不足导致反硝化失败", e);
            break;
        }

        let new_aerobic = calculate_aerobic_reactions(
            &new_anoxic, process, influent.temperature, dt_days, &kinetics
        );

        if let Err(e) = new_aerobic.check_valid() {
            success = false;
            error_msg = format!("好氧池计算异常: {}. 可能原因: 曝气量不足导致氨氮积累", e);
            break;
        }

        let (new_final, new_return_sludge) = calculate_clarifier(&new_aerobic, process);

        if let Err(e) = new_final.check_valid() {
            success = false;
            error_msg = format!("二沉池计算异常: {}", e);
            break;
        }

        anaerobic_eff = new_anaerobic.clone();
        anoxic_eff = new_anoxic.clone();
        aerobic_eff = new_aerobic.clone();
        final_eff = new_final.clone();
        return_sludge = new_return_sludge;
        internal_recirc = new_aerobic.clone();

        let sludge_production = process.daily_flow / 24.0 * sim_config.time_step_hours
            * (influent.cod - final_eff.cod) * 0.0005;
        let oxygen_demand = process.aeration_rate * sim_config.time_step_hours;

        time_series.push(TimeStepResult {
            time_hours,
            anaerobic_effluent: anaerobic_eff.clone(),
            anoxic_effluent: anoxic_eff.clone(),
            aerobic_effluent: aerobic_eff.clone(),
            final_effluent: final_eff.clone(),
            sludge_production,
            oxygen_demand,
        });
    }

    let unit_effluents = UnitEffluents {
        anaerobic: anaerobic_eff.clone(),
        anoxic: anoxic_eff.clone(),
        aerobic: aerobic_eff.clone(),
        clarifier: final_eff.clone(),
    };

    let compliance = check_compliance(&final_eff);
    let mass_balance = calculate_mass_balance(influent, &unit_effluents, process);
    let energy = calculate_energy(process, &final_eff);

    SimulationResult {
        success,
        message: error_msg,
        influent_warnings: warnings,
        time_series,
        final_effluent: final_eff,
        unit_effluents,
        compliance,
        mass_balance,
        energy_consumption: energy,
    }
}

pub fn check_compliance(effluent: &WaterQuality) -> ComplianceResult {
    let check = |value: f64, limit: f64| -> (f64, f64, bool) {
        (value, limit, value <= limit)
    };

    let cod = check(effluent.cod, 50.0);
    let bod5 = check(effluent.bod5, 10.0);
    let ss = check(effluent.ss, 10.0);
    let tn = check(effluent.tn, 15.0);
    let nh3_n = check(effluent.nh3_n, 5.0);
    let tp = check(effluent.tp, 0.5);

    let overall = cod.2 && bod5.2 && ss.2 && tn.2 && nh3_n.2 && tp.2;

    ComplianceResult { cod, bod5, ss, tn, nh3_n, tp, overall_compliant: overall }
}

pub fn calculate_mass_balance(
    influent: &InfluentParams,
    units: &UnitEffluents,
    process: &ProcessConfig,
) -> MassBalanceResult {
    let flow = process.daily_flow;
    let r = process.sludge_recirculation_ratio;
    let ri = process.internal_recirculation_ratio;

    let nitrogen_flows = vec![
        ("进水".to_string(), "厌氧池".to_string(), influent.tn * flow / 1000.0),
        ("回流污泥".to_string(), "厌氧池".to_string(), units.clarifier.tn * flow * r / 1000.0),
        ("厌氧池".to_string(), "缺氧池".to_string(), units.anaerobic.tn * flow * (1.0 + r) / 1000.0),
        ("内回流".to_string(), "缺氧池".to_string(), units.aerobic.tn * flow * ri / 1000.0),
        ("缺氧池".to_string(), "好氧池".to_string(), units.anoxic.tn * flow * (1.0 + r + ri) / 1000.0),
        ("好氧池".to_string(), "二沉池".to_string(), units.aerobic.tn * flow * (1.0 + r + ri) / 1000.0),
        ("好氧池".to_string(), "内回流".to_string(), units.aerobic.tn * flow * ri / 1000.0),
        ("二沉池".to_string(), "出水".to_string(), units.clarifier.tn * flow / 1000.0),
        ("二沉池".to_string(), "回流污泥".to_string(), units.clarifier.tn * flow * r / 1000.0),
        ("系统".to_string(), "N2排放".to_string(), (influent.tn - units.clarifier.tn) * flow / 1000.0),
    ];

    let phosphorus_flows = vec![
        ("进水".to_string(), "厌氧池".to_string(), influent.tp * flow / 1000.0),
        ("厌氧池".to_string(), "缺氧池".to_string(), units.anaerobic.tp * flow * (1.0 + r) / 1000.0),
        ("缺氧池".to_string(), "好氧池".to_string(), units.anoxic.tp * flow * (1.0 + r + ri) / 1000.0),
        ("好氧池".to_string(), "二沉池".to_string(), units.aerobic.tp * flow * (1.0 + r + ri) / 1000.0),
        ("二沉池".to_string(), "出水".to_string(), units.clarifier.tp * flow / 1000.0),
        ("二沉池".to_string(), "剩余污泥".to_string(), (influent.tp - units.clarifier.tp) * flow / 1000.0 * 0.8),
    ];

    let carbon_flows = vec![
        ("进水".to_string(), "厌氧池".to_string(), influent.cod * flow / 1000.0),
        ("厌氧池".to_string(), "缺氧池".to_string(), units.anaerobic.cod * flow * (1.0 + r) / 1000.0),
        ("缺氧池".to_string(), "好氧池".to_string(), units.anoxic.cod * flow * (1.0 + r + ri) / 1000.0),
        ("好氧池".to_string(), "二沉池".to_string(), units.aerobic.cod * flow * (1.0 + r + ri) / 1000.0),
        ("二沉池".to_string(), "出水".to_string(), units.clarifier.cod * flow / 1000.0),
        ("系统".to_string(), "CO2排放".to_string(), (influent.cod - units.clarifier.cod) * flow / 1000.0 * 0.7),
    ];

    MassBalanceResult { nitrogen_flows, phosphorus_flows, carbon_flows }
}

pub fn calculate_energy(process: &ProcessConfig, _effluent: &WaterQuality) -> EnergyResult {
    let air_pressure = 0.6;
    let blower_efficiency = 0.6;
    let aeration_kwh_per_hour = process.aeration_rate * air_pressure / (3600.0 * blower_efficiency) * 1.2;
    let aeration_kwh = aeration_kwh_per_hour * 24.0;

    let tank_volumes = process.anaerobic.volume + process.anoxic.volume + process.aerobic.volume;
    let mixing_power = tank_volumes * 5.0 / 1000.0;
    let mixing_kwh = mixing_power * 24.0;

    let r = process.sludge_recirculation_ratio;
    let return_flow_m3h = process.daily_flow * r / 24.0;
    let pump_head = 8.0;
    let pump_efficiency = 0.7;
    let pumping_kwh = return_flow_m3h * pump_head * 9.81 / (3600.0 * pump_efficiency) * 24.0;

    let total_kwh = aeration_kwh + mixing_kwh + pumping_kwh;
    let kwh_per_m3 = total_kwh / process.daily_flow;

    let breakdown = vec![
        ("曝气能耗".to_string(), aeration_kwh, aeration_kwh / total_kwh * 100.0),
        ("搅拌能耗".to_string(), mixing_kwh, mixing_kwh / total_kwh * 100.0),
        ("污泥回流泵能耗".to_string(), pumping_kwh, pumping_kwh / total_kwh * 100.0),
    ];

    EnergyResult {
        aeration_kwh,
        mixing_kwh,
        pumping_kwh,
        total_kwh,
        kwh_per_m3,
        breakdown,
    }
}
