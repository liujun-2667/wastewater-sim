use crate::models::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvRecord {
    pub date: String,
    pub cod: f64,
    pub bod5: f64,
    pub ss: f64,
    pub tn: f64,
    pub nh3_n: f64,
    pub tp: f64,
    pub ph: f64,
    pub temperature: f64,
}

pub fn parse_csv(csv_content: &str) -> Result<Vec<CsvDataPoint>, String> {
    let mut reader = csv::Reader::from_reader(csv_content.as_bytes());
    let mut points = Vec::new();

    for (idx, result) in reader.deserialize().enumerate() {
        let record: CsvRecord = match result {
            Ok(r) => r,
            Err(e) => return Err(format!("CSV解析错误，行{}: {}", idx + 2, e)),
        };

        points.push(CsvDataPoint {
            date: record.date.clone(),
            params: InfluentParams {
                cod: record.cod,
                bod5: record.bod5,
                ss: record.ss,
                tn: record.tn,
                nh3_n: record.nh3_n,
                tp: record.tp,
                ph: record.ph,
                temperature: record.temperature,
            },
        });
    }

    if points.is_empty() {
        return Err("CSV文件为空或格式不正确".to_string());
    }

    Ok(points)
}

pub fn compute_statistics(points: &[CsvDataPoint]) -> CsvStatistics {
    let n = points.len();

    let sum = |f: &dyn Fn(&InfluentParams) -> f64| -> f64 {
        points.iter().map(|p| f(&p.params)).sum()
    };

    let means = InfluentParams {
        cod: sum(&|p| p.cod) / n as f64,
        bod5: sum(&|p| p.bod5) / n as f64,
        ss: sum(&|p| p.ss) / n as f64,
        tn: sum(&|p| p.tn) / n as f64,
        nh3_n: sum(&|p| p.nh3_n) / n as f64,
        tp: sum(&|p| p.tp) / n as f64,
        ph: sum(&|p| p.ph) / n as f64,
        temperature: sum(&|p| p.temperature) / n as f64,
    };

    let mut maxes = points[0].params.clone();
    let mut mins = points[0].params.clone();
    for p in points.iter() {
        maxes.cod = maxes.cod.max(p.params.cod);
        maxes.bod5 = maxes.bod5.max(p.params.bod5);
        maxes.ss = maxes.ss.max(p.params.ss);
        maxes.tn = maxes.tn.max(p.params.tn);
        maxes.nh3_n = maxes.nh3_n.max(p.params.nh3_n);
        maxes.tp = maxes.tp.max(p.params.tp);
        maxes.ph = maxes.ph.max(p.params.ph);
        maxes.temperature = maxes.temperature.max(p.params.temperature);

        mins.cod = mins.cod.min(p.params.cod);
        mins.bod5 = mins.bod5.min(p.params.bod5);
        mins.ss = mins.ss.min(p.params.ss);
        mins.tn = mins.tn.min(p.params.tn);
        mins.nh3_n = mins.nh3_n.min(p.params.nh3_n);
        mins.tp = mins.tp.min(p.params.tp);
        mins.ph = mins.ph.min(p.params.ph);
        mins.temperature = mins.temperature.min(p.params.temperature);
    }

    let std_dev_fn = |f: &dyn Fn(&InfluentParams) -> f64, mean: f64| -> f64 {
        let variance: f64 = points.iter()
            .map(|p| (f(&p.params) - mean).powi(2))
            .sum::<f64>() / n as f64;
        variance.sqrt()
    };

    let std_devs = InfluentParams {
        cod: std_dev_fn(&|p| p.cod, means.cod),
        bod5: std_dev_fn(&|p| p.bod5, means.bod5),
        ss: std_dev_fn(&|p| p.ss, means.ss),
        tn: std_dev_fn(&|p| p.tn, means.tn),
        nh3_n: std_dev_fn(&|p| p.nh3_n, means.nh3_n),
        tp: std_dev_fn(&|p| p.tp, means.tp),
        ph: std_dev_fn(&|p| p.ph, means.ph),
        temperature: std_dev_fn(&|p| p.temperature, means.temperature),
    };

    let histograms = Histograms {
        cod: build_histogram(points, &|p| p.cod, mins.cod, maxes.cod, 10),
        bod5: build_histogram(points, &|p| p.bod5, mins.bod5, maxes.bod5, 10),
        ss: build_histogram(points, &|p| p.ss, mins.ss, maxes.ss, 10),
        tn: build_histogram(points, &|p| p.tn, mins.tn, maxes.tn, 10),
        nh3_n: build_histogram(points, &|p| p.nh3_n, mins.nh3_n, maxes.nh3_n, 10),
        tp: build_histogram(points, &|p| p.tp, mins.tp, maxes.tp, 10),
    };

    CsvStatistics {
        count: n,
        means,
        maxes,
        mins,
        std_devs,
        histograms,
    }
}

fn build_histogram(
    points: &[CsvDataPoint],
    f: &dyn Fn(&InfluentParams) -> f64,
    min: f64,
    max: f64,
    bins: usize,
) -> Vec<(f64, usize)> {
    let range = (max - min).max(0.001);
    let bin_width = range / bins as f64;
    let mut counts = vec![0usize; bins];

    for p in points {
        let val = f(&p.params);
        let mut idx = ((val - min) / bin_width) as usize;
        if idx >= bins { idx = bins - 1; }
        if idx < bins {
            counts[idx] += 1;
        }
    }

    let mut result = Vec::new();
    for i in 0..bins {
        let center = min + bin_width * (i as f64 + 0.5);
        result.push((center, counts[i]));
    }
    result
}
