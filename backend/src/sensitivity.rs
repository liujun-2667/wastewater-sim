use crate::models::*;
use crate::simulation::run_simulation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityRequest {
    pub param_name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub steps: usize,
    pub influent: InfluentParams,
    pub process_config: ProcessConfig,
    pub sim_config: SimulationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityPoint {
    pub param_value: f64,
    pub cod: f64,
    pub bod5: f64,
    pub ss: f64,
    pub tn: f64,
    pub nh3_n: f64,
    pub tp: f64,
    pub kwh_per_m3: f64,
    pub compliant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityResult {
    pub param_name: String,
    pub points: Vec<SensitivityPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoDimSensitivityRequest {
    pub param_x: String,
    pub param_y: String,
    pub x_min: f64,
    pub x_max: f64,
    pub x_steps: usize,
    pub y_min: f64,
    pub y_max: f64,
    pub y_steps: usize,
    pub target_param: String,
    pub influent: InfluentParams,
    pub process_config: ProcessConfig,
    pub sim_config: SimulationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoDimSensitivityResult {
    pub param_x: String,
    pub param_y: String,
    pub target_param: String,
    pub x_values: Vec<f64>,
    pub y_values: Vec<f64>,
    pub z_matrix: Vec<Vec<f64>>,
}

pub fn run_sensitivity_analysis(req: &SensitivityRequest) -> SensitivityResult {
    let mut points = Vec::new();
    let step_size = if req.steps > 1 {
        (req.max_value - req.min_value) / (req.steps - 1) as f64
    } else {
        0.0
    };

    for i in 0..req.steps {
        let param_value = req.min_value + step_size * i as f64;
        let mut process = req.process_config.clone();

        match req.param_name.as_str() {
            "aeration_rate" => process.aeration_rate = param_value,
            "sludge_recirculation_ratio" => process.sludge_recirculation_ratio = param_value,
            "internal_recirculation_ratio" => process.internal_recirculation_ratio = param_value,
            "srt" => process.srt = param_value,
            "mlss_aerobic" => process.aerobic.mlss = param_value,
            "mlss_anaerobic" => process.anaerobic.mlss = param_value,
            "mlss_anoxic" => process.anoxic.mlss = param_value,
            "do_aerobic" => process.aerobic.do_setpoint = param_value,
            "do_anaerobic" => process.anaerobic.do_setpoint = param_value,
            "do_anoxic" => process.anoxic.do_setpoint = param_value,
            "hrt_aerobic" => process.aerobic.hrt = param_value,
            "hrt_anaerobic" => process.anaerobic.hrt = param_value,
            "hrt_anoxic" => process.anoxic.hrt = param_value,
            "hrt_clarifier" => process.clarifier.hrt = param_value,
            _ => {}
        }

        let result = run_simulation(&req.influent, &process, &req.sim_config);

        points.push(SensitivityPoint {
            param_value,
            cod: result.final_effluent.cod,
            bod5: result.final_effluent.bod5,
            ss: result.final_effluent.ss,
            tn: result.final_effluent.tn,
            nh3_n: result.final_effluent.nh3_n,
            tp: result.final_effluent.tp,
            kwh_per_m3: result.energy_consumption.kwh_per_m3,
            compliant: result.compliance.overall_compliant,
        });
    }

    SensitivityResult {
        param_name: req.param_name.clone(),
        points,
    }
}

pub fn run_2d_sensitivity_analysis(req: &TwoDimSensitivityRequest) -> TwoDimSensitivityResult {
    let x_step = if req.x_steps > 1 {
        (req.x_max - req.x_min) / (req.x_steps - 1) as f64
    } else { 0.0 };
    let y_step = if req.y_steps > 1 {
        (req.y_max - req.y_min) / (req.y_steps - 1) as f64
    } else { 0.0 };

    let mut x_values = Vec::new();
    let mut y_values = Vec::new();
    let mut z_matrix: Vec<Vec<f64>> = Vec::new();

    for i in 0..req.x_steps {
        x_values.push(req.x_min + x_step * i as f64);
    }
    for j in 0..req.y_steps {
        y_values.push(req.y_min + y_step * j as f64);
    }

    for j in 0..req.y_steps {
        let mut row = Vec::new();
        for i in 0..req.x_steps {
            let mut process = req.process_config.clone();
            let x_val = req.x_min + x_step * i as f64;
            let y_val = req.y_min + y_step * j as f64;

            match req.param_x.as_str() {
                "aeration_rate" => process.aeration_rate = x_val,
                "sludge_recirculation_ratio" => process.sludge_recirculation_ratio = x_val,
                "internal_recirculation_ratio" => process.internal_recirculation_ratio = x_val,
                "srt" => process.srt = x_val,
                _ => {}
            }
            match req.param_y.as_str() {
                "aeration_rate" => process.aeration_rate = y_val,
                "sludge_recirculation_ratio" => process.sludge_recirculation_ratio = y_val,
                "internal_recirculation_ratio" => process.internal_recirculation_ratio = y_val,
                "srt" => process.srt = y_val,
                _ => {}
            }

            let result = run_simulation(&req.influent, &process, &req.sim_config);
            let z_val = match req.target_param.as_str() {
                "cod" => result.final_effluent.cod,
                "bod5" => result.final_effluent.bod5,
                "ss" => result.final_effluent.ss,
                "tn" => result.final_effluent.tn,
                "nh3_n" => result.final_effluent.nh3_n,
                "tp" => result.final_effluent.tp,
                "kwh_per_m3" => result.energy_consumption.kwh_per_m3,
                _ => 0.0,
            };
            row.push(z_val);
        }
        z_matrix.push(row);
    }

    TwoDimSensitivityResult {
        param_x: req.param_x.clone(),
        param_y: req.param_y.clone(),
        target_param: req.target_param.clone(),
        x_values,
        y_values,
        z_matrix,
    }
}
