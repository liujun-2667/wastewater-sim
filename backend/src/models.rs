use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfluentParams {
    pub cod: f64,
    pub bod5: f64,
    pub ss: f64,
    pub tn: f64,
    pub nh3_n: f64,
    pub tp: f64,
    pub ph: f64,
    pub temperature: f64,
}

impl InfluentParams {
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.cod < 100.0 || self.cod > 500.0 {
            warnings.push(format!("COD值{}mg/L超出典型范围100-500mg/L", self.cod));
        }
        if self.bod5 < 50.0 || self.bod5 > 300.0 {
            warnings.push(format!("BOD5值{}mg/L超出典型范围50-300mg/L", self.bod5));
        }
        if self.ss < 50.0 || self.ss > 300.0 {
            warnings.push(format!("SS值{}mg/L超出典型范围50-300mg/L", self.ss));
        }
        if self.tn < 20.0 || self.tn > 80.0 {
            warnings.push(format!("TN值{}mg/L超出典型范围20-80mg/L", self.tn));
        }
        if self.nh3_n < 10.0 || self.nh3_n > 50.0 {
            warnings.push(format!("NH3-N值{}mg/L超出典型范围10-50mg/L", self.nh3_n));
        }
        if self.tp < 2.0 || self.tp > 10.0 {
            warnings.push(format!("TP值{}mg/L超出典型范围2-10mg/L", self.tp));
        }
        if self.ph < 6.0 || self.ph > 9.0 {
            warnings.push(format!("pH值{}超出典型范围6-9", self.ph));
        }
        if self.temperature < 5.0 || self.temperature > 35.0 {
            warnings.push(format!("水温{}℃超出典型范围5-35℃", self.temperature));
        }
        warnings
    }

    pub fn default() -> Self {
        Self {
            cod: 300.0,
            bod5: 150.0,
            ss: 200.0,
            tn: 40.0,
            nh3_n: 25.0,
            tp: 5.0,
            ph: 7.2,
            temperature: 20.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitConfig {
    pub volume: f64,
    pub hrt: f64,
    pub mlss: f64,
    pub do_setpoint: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub anaerobic: UnitConfig,
    pub anoxic: UnitConfig,
    pub aerobic: UnitConfig,
    pub clarifier: UnitConfig,
    pub sludge_recirculation_ratio: f64,
    pub internal_recirculation_ratio: f64,
    pub srt: f64,
    pub aeration_rate: f64,
    pub daily_flow: f64,
}

impl ProcessConfig {
    pub fn small_plant() -> Self {
        Self {
            anaerobic: UnitConfig { volume: 800.0, hrt: 1.92, mlss: 3500.0, do_setpoint: 0.1 },
            anoxic: UnitConfig { volume: 1200.0, hrt: 2.88, mlss: 3500.0, do_setpoint: 0.3 },
            aerobic: UnitConfig { volume: 3000.0, hrt: 7.2, mlss: 3500.0, do_setpoint: 2.5 },
            clarifier: UnitConfig { volume: 1500.0, hrt: 3.6, mlss: 7000.0, do_setpoint: 0.0 },
            sludge_recirculation_ratio: 0.8,
            internal_recirculation_ratio: 2.0,
            srt: 15.0,
            aeration_rate: 600.0,
            daily_flow: 10000.0,
        }
    }

    pub fn medium_plant() -> Self {
        Self {
            anaerobic: UnitConfig { volume: 4000.0, hrt: 1.92, mlss: 4000.0, do_setpoint: 0.1 },
            anoxic: UnitConfig { volume: 6000.0, hrt: 2.88, mlss: 4000.0, do_setpoint: 0.3 },
            aerobic: UnitConfig { volume: 15000.0, hrt: 7.2, mlss: 4000.0, do_setpoint: 3.0 },
            clarifier: UnitConfig { volume: 7500.0, hrt: 3.6, mlss: 8000.0, do_setpoint: 0.0 },
            sludge_recirculation_ratio: 1.0,
            internal_recirculation_ratio: 2.5,
            srt: 18.0,
            aeration_rate: 3000.0,
            daily_flow: 50000.0,
        }
    }

    pub fn large_plant() -> Self {
        Self {
            anaerobic: UnitConfig { volume: 16000.0, hrt: 1.92, mlss: 4500.0, do_setpoint: 0.1 },
            anoxic: UnitConfig { volume: 24000.0, hrt: 2.88, mlss: 4500.0, do_setpoint: 0.3 },
            aerobic: UnitConfig { volume: 60000.0, hrt: 7.2, mlss: 4500.0, do_setpoint: 3.5 },
            clarifier: UnitConfig { volume: 30000.0, hrt: 3.6, mlss: 9000.0, do_setpoint: 0.0 },
            sludge_recirculation_ratio: 1.2,
            internal_recirculation_ratio: 3.0,
            srt: 20.0,
            aeration_rate: 12000.0,
            daily_flow: 200000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterQuality {
    pub cod: f64,
    pub bod5: f64,
    pub ss: f64,
    pub tn: f64,
    pub nh3_n: f64,
    pub no3_n: f64,
    pub tp: f64,
    pub po4_p: f64,
    pub vfa: f64,
    #[serde(rename = "do")]
    pub dissolved_oxygen: f64,
}

impl WaterQuality {
    pub fn from_influent(influent: &InfluentParams) -> Self {
        Self {
            cod: influent.cod,
            bod5: influent.bod5,
            ss: influent.ss,
            tn: influent.tn,
            nh3_n: influent.nh3_n,
            no3_n: (influent.tn - influent.nh3_n) * 0.1,
            tp: influent.tp,
            po4_p: influent.tp * 0.6,
            vfa: influent.cod * 0.15,
            dissolved_oxygen: 0.5,
        }
    }

    pub fn empty() -> Self {
        Self {
            cod: 0.0, bod5: 0.0, ss: 0.0, tn: 0.0, nh3_n: 0.0,
            no3_n: 0.0, tp: 0.0, po4_p: 0.0, vfa: 0.0, dissolved_oxygen: 0.0,
        }
    }

    pub fn check_valid(&self) -> Result<(), String> {
        let vals = [self.cod, self.bod5, self.ss, self.tn, self.nh3_n, self.no3_n, self.tp, self.po4_p];
        for &v in &vals {
            if v < -0.001 {
                return Err(format!("检测到负浓度值: {}", v));
            }
            if v.is_nan() || v.is_infinite() {
                return Err(format!("检测到无效数值: {}", v));
            }
            if v > 100000.0 {
                return Err(format!("检测到异常高浓度值: {}", v));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub time_step_hours: f64,
    pub total_duration_days: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            time_step_hours: 1.0,
            total_duration_days: 7.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvDataPoint {
    pub date: String,
    pub params: InfluentParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvStatistics {
    pub count: usize,
    pub means: InfluentParams,
    pub maxes: InfluentParams,
    pub mins: InfluentParams,
    pub std_devs: InfluentParams,
    pub histograms: Histograms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histograms {
    pub cod: Vec<(f64, usize)>,
    pub bod5: Vec<(f64, usize)>,
    pub ss: Vec<(f64, usize)>,
    pub tn: Vec<(f64, usize)>,
    pub nh3_n: Vec<(f64, usize)>,
    pub tp: Vec<(f64, usize)>,
}
