//! Operator-facing names for measurements, valves, disturbances, and setpoints.

use serde::Serialize;

use crate::closed_loop::PlantWideController;
use crate::process::{default_delta_t, DEFAULT_RNG_SEED, N_IDV, N_XMEAS, N_XMV};

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IdvKind {
    Step,
    RandomVariation,
    SlowDrift,
    Sticking,
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
pub struct MeasMeta {
    pub n: usize,
    pub name_en: &'static str,
    pub name_zh: &'static str,
    pub unit: &'static str,
    pub group_zh: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct MvMeta {
    pub n: usize,
    pub name_en: &'static str,
    pub name_zh: &'static str,
    pub unit: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct IdvMeta {
    pub n: usize,
    pub name_en: &'static str,
    pub name_zh: &'static str,
    pub kind: IdvKind,
    pub kind_zh: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct SetpointMeta {
    pub n: usize,
    pub name_en: &'static str,
    pub name_zh: &'static str,
    pub unit: &'static str,
    pub pv: &'static str,
    pub default: f64,
    /// Outer loop that writes another setpoint rather than an `XMV`.
    pub cascade: bool,
    pub group_zh: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct Catalog {
    pub delta_t_hours: f64,
    pub default_seed: f64,
    pub n_xmeas: usize,
    pub n_xmv: usize,
    pub n_idv: usize,
    pub n_setpoints: usize,
    pub xmeas: Vec<MeasMeta>,
    pub xmv: Vec<MvMeta>,
    pub idv: Vec<IdvMeta>,
    pub setpoints: Vec<SetpointMeta>,
}

pub fn catalog() -> Catalog {
    let defaults = PlantWideController::default_setpoints();
    Catalog {
        delta_t_hours: default_delta_t(),
        default_seed: DEFAULT_RNG_SEED,
        n_xmeas: N_XMEAS,
        n_xmv: N_XMV,
        n_idv: N_IDV,
        n_setpoints: 20,
        xmeas: xmeas_meta(),
        xmv: xmv_meta(),
        idv: idv_meta(),
        setpoints: setpoint_meta(&defaults),
    }
}

fn xmeas_meta() -> Vec<MeasMeta> {
    [
        (1, "A Feed (stream 1)", "A 进料（物流 1）", "kscmh", "流量"),
        (2, "D Feed (stream 2)", "D 进料（物流 2）", "kg/h", "流量"),
        (3, "E Feed (stream 3)", "E 进料（物流 3）", "kg/h", "流量"),
        (
            4,
            "A and C Feed (stream 4)",
            "A/C 进料（物流 4）",
            "kscmh",
            "流量",
        ),
        (
            5,
            "Recycle Flow (stream 8)",
            "循环流量（物流 8）",
            "kscmh",
            "流量",
        ),
        (
            6,
            "Reactor Feed Rate (stream 6)",
            "反应器进料（物流 6）",
            "kscmh",
            "反应器",
        ),
        (7, "Reactor Pressure", "反应器压力", "kPa gauge", "反应器"),
        (8, "Reactor Level", "反应器液位", "%", "反应器"),
        (9, "Reactor Temperature", "反应器温度", "°C", "反应器"),
        (
            10,
            "Purge Rate (stream 9)",
            "放空流量（物流 9）",
            "kscmh",
            "流量",
        ),
        (11, "Product Sep Temp", "分离器温度", "°C", "分离器"),
        (12, "Product Sep Level", "分离器液位", "%", "分离器"),
        (13, "Prod Sep Pressure", "分离器压力", "kPa gauge", "分离器"),
        (
            14,
            "Prod Sep Underflow (stream 10)",
            "分离器釜液（物流 10）",
            "m³/h",
            "分离器",
        ),
        (15, "Stripper Level", "汽提塔液位", "%", "汽提塔"),
        (16, "Stripper Pressure", "汽提塔压力", "kPa gauge", "汽提塔"),
        (
            17,
            "Stripper Underflow (stream 11)",
            "产品流量（物流 11）",
            "m³/h",
            "汽提塔",
        ),
        (18, "Stripper Temperature", "汽提塔温度", "°C", "汽提塔"),
        (19, "Stripper Steam Flow", "汽提蒸汽流量", "kg/h", "汽提塔"),
        (20, "Compressor Work", "压缩机功率", "kW", "压缩机"),
        (
            21,
            "Reactor Cooling Water Outlet Temp",
            "反应器冷却水出口温度",
            "°C",
            "反应器",
        ),
        (
            22,
            "Separator Cooling Water Outlet Temp",
            "冷凝器冷却水出口温度",
            "°C",
            "分离器",
        ),
        (
            23,
            "Component A (stream 6)",
            "组分 A（物流 6）",
            "mol %",
            "成分·进料",
        ),
        (
            24,
            "Component B (stream 6)",
            "组分 B（物流 6）",
            "mol %",
            "成分·进料",
        ),
        (
            25,
            "Component C (stream 6)",
            "组分 C（物流 6）",
            "mol %",
            "成分·进料",
        ),
        (
            26,
            "Component D (stream 6)",
            "组分 D（物流 6）",
            "mol %",
            "成分·进料",
        ),
        (
            27,
            "Component E (stream 6)",
            "组分 E（物流 6）",
            "mol %",
            "成分·进料",
        ),
        (
            28,
            "Component F (stream 6)",
            "组分 F（物流 6）",
            "mol %",
            "成分·进料",
        ),
        (
            29,
            "Component A (stream 9)",
            "组分 A（物流 9）",
            "mol %",
            "成分·放空",
        ),
        (
            30,
            "Component B (stream 9)",
            "组分 B（物流 9）",
            "mol %",
            "成分·放空",
        ),
        (
            31,
            "Component C (stream 9)",
            "组分 C（物流 9）",
            "mol %",
            "成分·放空",
        ),
        (
            32,
            "Component D (stream 9)",
            "组分 D（物流 9）",
            "mol %",
            "成分·放空",
        ),
        (
            33,
            "Component E (stream 9)",
            "组分 E（物流 9）",
            "mol %",
            "成分·放空",
        ),
        (
            34,
            "Component F (stream 9)",
            "组分 F（物流 9）",
            "mol %",
            "成分·放空",
        ),
        (
            35,
            "Component G (stream 9)",
            "组分 G（物流 9）",
            "mol %",
            "成分·放空",
        ),
        (
            36,
            "Component H (stream 9)",
            "组分 H（物流 9）",
            "mol %",
            "成分·放空",
        ),
        (
            37,
            "Component D (stream 11)",
            "组分 D（物流 11）",
            "mol %",
            "成分·产品",
        ),
        (
            38,
            "Component E (stream 11)",
            "组分 E（物流 11）",
            "mol %",
            "成分·产品",
        ),
        (
            39,
            "Component F (stream 11)",
            "组分 F（物流 11）",
            "mol %",
            "成分·产品",
        ),
        (
            40,
            "Component G (stream 11)",
            "组分 G（物流 11）",
            "mol %",
            "成分·产品",
        ),
        (
            41,
            "Component H (stream 11)",
            "组分 H（物流 11）",
            "mol %",
            "成分·产品",
        ),
    ]
    .into_iter()
    .map(|(n, name_en, name_zh, unit, group_zh)| MeasMeta {
        n,
        name_en,
        name_zh,
        unit,
        group_zh,
    })
    .collect()
}

fn xmv_meta() -> Vec<MvMeta> {
    [
        (1, "D Feed Flow (stream 2)", "D 进料阀（物流 2）"),
        (2, "E Feed Flow (stream 3)", "E 进料阀（物流 3）"),
        (3, "A Feed Flow (stream 1)", "A 进料阀（物流 1）"),
        (4, "A and C Feed Flow (stream 4)", "A/C 进料阀（物流 4）"),
        (5, "Compressor Recycle Valve", "压缩机循环阀"),
        (6, "Purge Valve (stream 9)", "放空阀（物流 9）"),
        (
            7,
            "Separator Pot Liquid Flow (stream 10)",
            "分离器釜液阀（物流 10）",
        ),
        (
            8,
            "Stripper Liquid Product Flow (stream 11)",
            "产品阀（物流 11）",
        ),
        (9, "Stripper Steam Valve", "汽提蒸汽阀"),
        (10, "Reactor Cooling Water Flow", "反应器冷却水阀"),
        (11, "Condenser Cooling Water Flow", "冷凝器冷却水阀"),
        (12, "Agitator Speed", "搅拌转速"),
    ]
    .into_iter()
    .map(|(n, name_en, name_zh)| MvMeta {
        n,
        name_en,
        name_zh,
        unit: "%",
    })
    .collect()
}

fn idv_meta() -> Vec<IdvMeta> {
    [
        (
            1,
            "A/C Feed Ratio, B Composition Constant (Stream 4)",
            "物流 4 的 A/C 比阶跃，B 含量不变",
            IdvKind::Step,
            "阶跃",
        ),
        (
            2,
            "B Composition, A/C Ratio Constant (Stream 4)",
            "物流 4 的 B 含量阶跃，A/C 比不变",
            IdvKind::Step,
            "阶跃",
        ),
        (
            3,
            "D Feed Temperature (Stream 2)",
            "物流 2（D）进料温度阶跃",
            IdvKind::Step,
            "阶跃",
        ),
        (
            4,
            "Reactor Cooling Water Inlet Temperature",
            "反应器冷却水入口温度阶跃",
            IdvKind::Step,
            "阶跃",
        ),
        (
            5,
            "Condenser Cooling Water Inlet Temperature",
            "冷凝器冷却水入口温度阶跃",
            IdvKind::Step,
            "阶跃",
        ),
        (
            6,
            "A Feed Loss (Stream 1)",
            "物流 1（A）进料损失",
            IdvKind::Step,
            "阶跃",
        ),
        (
            7,
            "C Header Pressure Loss (Stream 4)",
            "物流 4 的 C 进料压力损失",
            IdvKind::Step,
            "阶跃",
        ),
        (
            8,
            "A, B, C Feed Composition (Stream 4)",
            "物流 4 的 A/B/C 组成随机变化",
            IdvKind::RandomVariation,
            "随机变化",
        ),
        (
            9,
            "D Feed Temperature (Stream 2)",
            "物流 2（D）进料温度随机变化",
            IdvKind::RandomVariation,
            "随机变化",
        ),
        (
            10,
            "C Feed Temperature (Stream 4)",
            "物流 4（C）进料温度随机变化",
            IdvKind::RandomVariation,
            "随机变化",
        ),
        (
            11,
            "Reactor Cooling Water Inlet Temperature",
            "反应器冷却水入口温度随机变化",
            IdvKind::RandomVariation,
            "随机变化",
        ),
        (
            12,
            "Condenser Cooling Water Inlet Temperature",
            "冷凝器冷却水入口温度随机变化",
            IdvKind::RandomVariation,
            "随机变化",
        ),
        (
            13,
            "Reaction Kinetics",
            "反应动力学缓慢漂移",
            IdvKind::SlowDrift,
            "缓慢漂移",
        ),
        (
            14,
            "Reactor Cooling Water Valve",
            "反应器冷却水阀门卡涩",
            IdvKind::Sticking,
            "卡涩",
        ),
        (
            15,
            "Condenser Cooling Water Valve",
            "冷凝器冷却水阀门卡涩",
            IdvKind::Sticking,
            "卡涩",
        ),
        (16, "Unknown", "未知", IdvKind::Unknown, "未知"),
        (17, "Unknown", "未知", IdvKind::Unknown, "未知"),
        (18, "Unknown", "未知", IdvKind::Unknown, "未知"),
        (19, "Unknown", "未知", IdvKind::Unknown, "未知"),
        (20, "Unknown", "未知", IdvKind::Unknown, "未知"),
    ]
    .into_iter()
    .map(|(n, name_en, name_zh, kind, kind_zh)| IdvMeta {
        n,
        name_en,
        name_zh,
        kind,
        kind_zh,
    })
    .collect()
}

fn setpoint_meta(defaults: &[f64; 20]) -> Vec<SetpointMeta> {
    [
        (
            1,
            "D Feed Flow",
            "D 进料流量",
            "kg/h",
            "XMEAS(2)",
            false,
            "内环",
        ),
        (
            2,
            "E Feed Flow",
            "E 进料流量",
            "kg/h",
            "XMEAS(3)",
            false,
            "内环",
        ),
        (
            3,
            "A Feed Flow",
            "A 进料流量",
            "kscmh",
            "XMEAS(1)",
            false,
            "内环",
        ),
        (
            4,
            "A/C Feed Flow",
            "A/C 进料流量",
            "kscmh",
            "XMEAS(4)",
            false,
            "内环",
        ),
        (
            5,
            "Recycle Flow",
            "循环流量",
            "kscmh",
            "XMEAS(5)",
            false,
            "内环",
        ),
        (
            6,
            "Purge Rate",
            "放空流量",
            "kscmh",
            "XMEAS(10)",
            false,
            "内环",
        ),
        (
            7,
            "Separator Level",
            "分离器液位",
            "%",
            "XMEAS(12)",
            false,
            "装置",
        ),
        (
            8,
            "Stripper Level",
            "汽提塔液位",
            "%",
            "XMEAS(15)",
            false,
            "装置",
        ),
        (
            9,
            "Stripper Steam Flow",
            "汽提蒸汽流量",
            "kg/h",
            "XMEAS(19)",
            false,
            "内环",
        ),
        (
            10,
            "Reactor CW Outlet Temp",
            "反应器冷却水出口温度",
            "°C",
            "XMEAS(21)",
            false,
            "内环",
        ),
        (
            11,
            "Stripper Underflow",
            "产品流量",
            "m³/h",
            "XMEAS(17)",
            false,
            "内环",
        ),
        (
            12,
            "Separator Pressure (unused loop)",
            "分离器压力（未接入主回路）",
            "kPa gauge",
            "XMEAS(13)",
            false,
            "备用",
        ),
        (
            13,
            "Reactor Feed A",
            "反应器进料 A",
            "mol %",
            "XMEAS(23)",
            true,
            "装置",
        ),
        (
            14,
            "Reactor Feed D",
            "反应器进料 D",
            "mol %",
            "XMEAS(26)",
            true,
            "装置",
        ),
        (
            15,
            "Reactor Feed E",
            "反应器进料 E",
            "mol %",
            "XMEAS(27)",
            true,
            "装置",
        ),
        (
            16,
            "Stripper Temperature",
            "汽提塔温度",
            "°C",
            "XMEAS(18)",
            true,
            "装置",
        ),
        (
            17,
            "Reactor Level",
            "反应器液位",
            "%",
            "XMEAS(8)",
            true,
            "装置",
        ),
        (
            18,
            "Reactor Temperature",
            "反应器温度",
            "°C",
            "XMEAS(9)",
            true,
            "装置",
        ),
        (
            19,
            "Purge B Composition",
            "放空 B 含量",
            "mol %",
            "XMEAS(30)",
            true,
            "装置",
        ),
        (
            20,
            "Product E Composition",
            "产品 E 含量",
            "mol %",
            "XMEAS(38)",
            true,
            "装置",
        ),
    ]
    .into_iter()
    .map(
        |(n, name_en, name_zh, unit, pv, cascade, group_zh)| SetpointMeta {
            n,
            name_en,
            name_zh,
            unit,
            pv,
            default: defaults[n - 1],
            cascade,
            group_zh,
        },
    )
    .collect()
}
