use crate::models::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisRequest {
    pub influent: InfluentParams,
    pub process_config: ProcessConfig,
    pub final_effluent: WaterQuality,
    pub unit_effluents: UnitEffluents,
    pub compliance: ComplianceResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisConclusion {
    pub category: String,
    pub severity: String,
    pub description: String,
    pub related_param: String,
    pub current_value: f64,
    pub recommended_value: f64,
    pub recommendation_detail: String,
    pub priority: i32,
    pub param_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResponse {
    pub conclusions: Vec<DiagnosisConclusion>,
}

fn calc_severity(actual: f64, limit: f64) -> String {
    if limit <= 0.0 {
        return "中度".to_string();
    }
    let ratio = (actual - limit) / limit;
    if ratio < 0.2 {
        "轻度".to_string()
    } else if ratio <= 0.5 {
        "中度".to_string()
    } else {
        "重度".to_string()
    }
}

fn clamp_recommendation(value: f64, min_val: f64, max_val: f64) -> f64 {
    value.clamp(min_val, max_val)
}

fn total_hrt(process: &ProcessConfig) -> f64 {
    process.anaerobic.hrt + process.anoxic.hrt + process.aerobic.hrt + process.clarifier.hrt
}

pub fn run_diagnosis(req: &DiagnosisRequest) -> DiagnosisResponse {
    let mut conclusions: Vec<DiagnosisConclusion> = Vec::new();
    let eff = &req.final_effluent;
    let comp = &req.compliance;
    let units = &req.unit_effluents;
    let process = &req.process_config;
    let influent = &req.influent;

    let nh3_n_exceeded = !comp.nh3_n.2;
    let tn_exceeded = !comp.tn.2;
    let tp_exceeded = !comp.tp.2;
    let cod_exceeded = !comp.cod.2;
    let ss_exceeded = !comp.ss.2;

    let nh3_n_limit = comp.nh3_n.1;
    let tn_limit = comp.tn.1;
    let tp_limit = comp.tp.1;
    let cod_limit = comp.cod.1;
    let ss_limit = comp.ss.1;

    let q = process.daily_flow / 24.0;

    if nh3_n_exceeded && units.aerobic.dissolved_oxygen < 2.0 {
        let severity = calc_severity(comp.nh3_n.0, nh3_n_limit);
        let current_removal_rate = if influent.nh3_n > 0.0 {
            (influent.nh3_n - eff.nh3_n) / influent.nh3_n
        } else {
            0.0
        };
        let target_removal_rate = if influent.nh3_n > 0.0 {
            (influent.nh3_n - nh3_n_limit * 0.8) / influent.nh3_n
        } else {
            0.9
        };
        let k_nh = 0.4;
        let v_aerobic = process.aerobic.volume;
        let do_increase = (target_removal_rate - current_removal_rate) * k_nh * v_aerobic / q;
        let aeration_increase = do_increase * process.aerobic.volume * 0.25;
        let max_aeration = process.aeration_rate * 2.0;
        let recommended_aeration = clamp_recommendation(
            process.aeration_rate + aeration_increase,
            process.aeration_rate,
            max_aeration,
        );

        conclusions.push(DiagnosisConclusion {
            category: "硝化".to_string(),
            severity,
            description: "曝气量不足导致硝化受抑制".to_string(),
            related_param: "曝气量 (m³/h)".to_string(),
            current_value: process.aeration_rate,
            recommended_value: recommended_aeration,
            recommendation_detail: format!(
                "目标NH3-N去除率{:.1}% - 当前去除率{:.1}% = 差值{:.1}%, 乘以K_nh({})×V_aerobic({})/Q({:.1}), 曝气量增量{:.1} m³/h",
                target_removal_rate * 100.0,
                current_removal_rate * 100.0,
                (target_removal_rate - current_removal_rate) * 100.0,
                k_nh, v_aerobic, q, aeration_increase
            ),
            priority: 1,
            param_path: "aeration_rate".to_string(),
        });
    }

    if tn_exceeded && process.internal_recirculation_ratio < 2.0 {
        let severity = calc_severity(comp.tn.0, tn_limit);
        let ri_increase = if (influent.tn - eff.tn).abs() > 0.01 {
            (eff.tn - tn_limit) / (influent.tn - eff.tn) * 1.2
        } else {
            1.0
        };
        let recommended_ri = clamp_recommendation(
            process.internal_recirculation_ratio + ri_increase,
            process.internal_recirculation_ratio,
            5.0,
        );

        conclusions.push(DiagnosisConclusion {
            category: "反硝化".to_string(),
            severity,
            description: "内回流不足导致反硝化效率低".to_string(),
            related_param: "内回流比".to_string(),
            current_value: process.internal_recirculation_ratio,
            recommended_value: recommended_ri,
            recommendation_detail: format!(
                "内回流比增量 = (出水TN({:.2}) - 标准限值({})) / (进水TN({}) - 出水TN({:.2})) × 安全系数1.2 = {:.2}",
                eff.tn, tn_limit, influent.tn, eff.tn, ri_increase
            ),
            priority: 2,
            param_path: "internal_recirculation_ratio".to_string(),
        });
    }

    if tn_exceeded && units.anoxic.dissolved_oxygen > 0.5 {
        let severity = calc_severity(comp.tn.0, tn_limit);
        let recommended_do = 0.3;
        conclusions.push(DiagnosisConclusion {
            category: "反硝化".to_string(),
            severity,
            description: "缺氧池溶氧偏高抑制反硝化".to_string(),
            related_param: "缺氧池DO设定值 (mg/L)".to_string(),
            current_value: process.anoxic.do_setpoint,
            recommended_value: recommended_do,
            recommendation_detail: format!(
                "当前缺氧池DO为{:.2} mg/L (>0.5), 建议下调至{:.2} mg/L以恢复反硝化环境",
                units.anoxic.dissolved_oxygen, recommended_do
            ),
            priority: 2,
            param_path: "anoxic.do_setpoint".to_string(),
        });
    }

    if tp_exceeded && units.anaerobic.dissolved_oxygen > 0.2 {
        let severity = calc_severity(comp.tp.0, tp_limit);
        let recommended_anaerobic_do = 0.05;
        let current_hrt = process.anaerobic.hrt;
        let recommended_hrt = if current_hrt < 2.0 { 2.5 } else { current_hrt };
        let new_total_hrt = total_hrt(process) - current_hrt + recommended_hrt;
        let final_hrt = if new_total_hrt > 24.0 {
            current_hrt + (24.0 - total_hrt(process)).max(0.0)
        } else {
            recommended_hrt
        };

        conclusions.push(DiagnosisConclusion {
            category: "除磷".to_string(),
            severity,
            description: "厌氧池溶氧过高抑制释磷".to_string(),
            related_param: "厌氧池DO设定值 (mg/L)".to_string(),
            current_value: process.anaerobic.do_setpoint,
            recommended_value: recommended_anaerobic_do,
            recommendation_detail: format!(
                "当前厌氧池DO为{:.2} mg/L (>0.2), 建议下调至{} mg/L; 同时建议增大厌氧池HRT从{:.2}h至{:.2}h",
                units.anaerobic.dissolved_oxygen, recommended_anaerobic_do, current_hrt, final_hrt
            ),
            priority: 1,
            param_path: "anaerobic.do_setpoint".to_string(),
        });

        if final_hrt > current_hrt {
            conclusions.push(DiagnosisConclusion {
                category: "除磷".to_string(),
                severity: severity.clone(),
                description: "厌氧池HRT不足影响释磷".to_string(),
                related_param: "厌氧池HRT (h)".to_string(),
                current_value: current_hrt,
                recommended_value: final_hrt,
                recommendation_detail: format!(
                    "增大厌氧池HRT从{:.2}h至{:.2}h, 调整后总HRT={:.2}h (限制24h)",
                    current_hrt, final_hrt, new_total_hrt.min(24.0)
                ),
                priority: 1,
                param_path: "anaerobic.hrt".to_string(),
            });
        }
    }

    if cod_exceeded && process.srt < 10.0 {
        let severity = calc_severity(comp.cod.0, cod_limit);
        let current_removal_rate = if influent.cod > 0.0 {
            (influent.cod - eff.cod) / influent.cod
        } else {
            0.0
        };
        let target_removal_rate = if influent.cod > 0.0 {
            (influent.cod - cod_limit * 0.8) / influent.cod
        } else {
            0.95
        };
        let mu_max = 6.0;
        let srt_increase = if current_removal_rate > 0.0 && target_removal_rate > current_removal_rate {
            (target_removal_rate / current_removal_rate).ln() / mu_max
        } else {
            5.0
        };
        let recommended_srt = clamp_recommendation(
            process.srt + srt_increase,
            process.srt,
            30.0,
        );

        conclusions.push(DiagnosisConclusion {
            category: "有机物降解".to_string(),
            severity,
            description: "污泥龄过短有机物降解不充分".to_string(),
            related_param: "污泥龄SRT (天)".to_string(),
            current_value: process.srt,
            recommended_value: recommended_srt,
            recommendation_detail: format!(
                "SRT增量 = ln(目标去除率{:.1}%/当前去除率{:.1}%)/μ_max({}) = {:.2}天",
                target_removal_rate * 100.0, current_removal_rate * 100.0, mu_max, srt_increase
            ),
            priority: 3,
            param_path: "srt".to_string(),
        });
    }

    if ss_exceeded && process.clarifier.hrt < 2.0 {
        let severity = calc_severity(comp.ss.0, ss_limit);
        let settling_rate = 0.15;
        let target_ss = ss_limit * 0.8;
        let hrt_increase = if influent.ss > 0.0 {
            (target_ss - eff.ss) / (influent.ss * settling_rate)
        } else {
            1.0
        };
        let hrt_increase = hrt_increase.max(0.5);
        let recommended_hrt = process.clarifier.hrt + hrt_increase;
        let new_total_hrt = total_hrt(process) + hrt_increase;
        let final_hrt = if new_total_hrt > 24.0 {
            process.clarifier.hrt + (24.0 - total_hrt(process)).max(0.0)
        } else {
            recommended_hrt
        };

        conclusions.push(DiagnosisConclusion {
            category: "固液分离".to_string(),
            severity,
            description: "二沉池停留时间不足泥水分离效果差".to_string(),
            related_param: "二沉池HRT (h)".to_string(),
            current_value: process.clarifier.hrt,
            recommended_value: final_hrt,
            recommendation_detail: format!(
                "HRT增量 = (目标SS({:.2}) - 当前SS({:.2})) / (进水SS({}) × 沉降速率常数{}) = {:.2}h",
                target_ss, eff.ss, influent.ss, settling_rate, hrt_increase
            ),
            priority: 4,
            param_path: "clarifier.hrt".to_string(),
        });
    }

    conclusions.sort_by(|a, b| a.priority.cmp(&b.priority));

    DiagnosisResponse { conclusions }
}
