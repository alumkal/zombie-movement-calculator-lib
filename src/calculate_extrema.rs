use crate::common::*;
use num_traits::ToPrimitive;
use rayon::prelude::*;

// [(时间，是否减速)]
fn walk_time_impl(ice_time: &[i64], time: i64, freeze_time: (i64, i64)) -> Vec<(i64, bool)> {
    let mut ice_time: Vec<i64> = ice_time.iter().map(|x| x - 1).filter(|x| *x >= 0 && *x < time).collect();
    ice_time.sort_unstable();
    ice_time.push(time);
    let mut result: Vec<(i64, bool)> = vec![];
    let (mut last_ice, mut last_freeze) = (-1, 0);
    for t in ice_time {
        if last_ice < 0 {
            result.push((t, false));
            last_freeze = freeze_time.0;
        } else if t - last_ice < 1999 {
            result.push((t - last_ice - last_freeze, true));
            last_freeze = freeze_time.1;
        } else {
            result.push((1999 - last_freeze, true));
            result.push((t - last_ice - 1999, false));
            last_freeze = freeze_time.0;
        }
        last_ice = t;
    }
    let mut result2: Vec<(i64, bool)> = vec![];
    for (t, chill) in result {
        if t == 0 {
            continue;
        }
        if result2.is_empty() {
            result2.push((t, chill));
            continue;
        }
        let last = result2.len() - 1;
        if result2[last].1 == chill {
            result2[last].0 += t;
        } else {
            result2.push((t, chill));
        }
    }
    result2
}

fn walk_time(data: &ZombieData, ice_time: &[i64], time: i64) -> (Vec<(i64, bool)>, Vec<(i64, bool)>) {
    if data.chill_immune { (vec![(time, false)], vec![(time, false)]) }
    else if data.freeze_immune { (walk_time_impl(ice_time, time, (0, 0)), walk_time_impl(ice_time, time, (0, 0))) }
    else { (walk_time_impl(ice_time, time, (399, 299)), walk_time_impl(ice_time, time, (599, 399))) }
}

fn calculate_constant(data: &ZombieData, ice_time: &[i64], time: i64) -> (f64, f64) {
    let speed_min_norm = (data.speed.0 * 16384).round() / 16384;
    let speed_max_norm = (data.speed.1 * 16384).round() / 16384;
    let speed_min_chill = (data.speed.0 * Num::new(2, 5) * 16384).round() / 16384;
    let speed_max_chill = (data.speed.1 * Num::new(2, 5) * 16384).round() / 16384;
    let (mut x_min, mut x_max) = (Num::new(data.spawn.0, 1), Num::new(data.spawn.1, 1));
    let walk_time_ = walk_time(data, ice_time, time);
    for (t, chill) in walk_time_.0 {
        x_min -= (if chill { speed_max_chill } else { speed_max_norm }) * t;
    }
    for (t, chill) in walk_time_.1 {
        x_max -= (if chill { speed_min_chill } else { speed_min_norm }) * t;
    }
    (x_min.to_f64().unwrap(), x_max.to_f64().unwrap())
}

// 返回分母 <=n 且在 (l, r) 之间的所有分数，外加 l 和 r
fn fraction_between(n: i64, l: Num, r: Num) -> Vec<Num> {
    let mut result = vec![l];
    for i in 1..=n {
        let den_l = l * i;
        let den_r = r * i;
        let den_l = if den_l.is_integer() {den_l + 1} else {den_l.ceil()};
        let den_r = if den_r.is_integer() {den_r - 1} else {den_r.floor()};
        for j in den_l.to_integer()..=den_r.to_integer() {
            if num_integer::gcd(i, j) == 1 {
                result.push(Num::new(j, i));
            }
        }
    }
    result.push(r);
    result.sort_unstable();
    result
}

// arr[x0] + arr[x0 + k] + ... + arr[x0 + (n - 1) * k]
fn total_shift(arr: &Vec<Num>, n: i64, k: Num, x0: Num) -> Num {
    let n = Num::new(n, 1);
    let mut result = Num::new(0, 1);
    let first = x0.floor().to_integer();
    let last = (x0 + (n - 1) * k).floor().to_integer();
    let mut cur = Num::new(0, 1);
    for i in first..last {
        let next = ((Num::new(i + 1, 1) - x0) / k).ceil();
        result += arr[i as usize % arr.len()] * (next - cur);
        cur = next;
    }
    result + arr[last as usize % arr.len()] * (n - cur)
}

fn calculate_animation_impl(data: &ZombieData, walk_time: &[(i64, bool)], animation: Option<&Vec<Num>>) -> (f64, f64) {
    let animation = animation.unwrap_or_else(|| match &data.movement_type {
        MovementType::Animation(x) | MovementType::Dancing(x) => x,
        _ => unreachable!()
    });
    let anim_len = animation.len() as i64;
    let total: Num = animation.iter().sum();
    let speed_scale_factor = Num::new(47, 100) * anim_len / total;
    let dis_scale_factor = Num::new(anim_len + 1, anim_len);
    let n: i64 = walk_time.iter().map(|(t, chill)| if *chill { *t } else { *t * 2 }).sum();
    // k 是减速状态下相位的变化率
    let k_min = data.speed.0 * speed_scale_factor / 2;
    let k_max = data.speed.1 * speed_scale_factor / 2;
    // k 在 [k_segments[i], k_segments[i+1]) 范围内变化时 dx 正比于 k
    let k_segments = fraction_between(n, k_min, k_max);
    let (x_min, x_max) = k_segments
        .par_windows(2)
        .map(|lr| {
        let (l, r) = (lr[0], lr[1]);
        let shift_norm_l: Vec<_> = animation.iter().map(|x| (x * dis_scale_factor * l * 32768).round() / 16384).collect();
        let shift_norm_r: Vec<_> = animation.iter().map(|x| (x * dis_scale_factor * r * 32768).round() / 16384).collect();
        let shift_chill_l: Vec<_> = animation.iter().map(|x| (x * dis_scale_factor * l * 16384).round() / 16384).collect();
        let shift_chill_r: Vec<_> = animation.iter().map(|x| (x * dis_scale_factor * r * 16384).round() / 16384).collect();
        let (mut x_min, mut x_max) = (Num::new(data.spawn.0, 1), Num::new(data.spawn.1, 1));
        let mut phase = l * 2;
        for (t, chill) in walk_time {
            if *chill {
                x_min -= total_shift(&shift_chill_r, *t, l, phase);
                x_max -= total_shift(&shift_chill_l, *t, l, phase);
                phase += l * t;
            } else {
                x_min -= total_shift(&shift_norm_r, *t, l * 2, phase);
                x_max -= total_shift(&shift_norm_l, *t, l * 2, phase);
                phase += l * 2 * t;
            }
        }
        (x_min, x_max)
    }).reduce(|| { (Num::new(1000, 1), Num::new(0, 1)) },
        |(x_min1, x_max1), (x_min2, x_max2)| {
        (min(x_min1, x_min2), max(x_max1, x_max2))
    });
    (x_min.to_f64().unwrap(), x_max.to_f64().unwrap())
}

fn calculate_animation(data: &ZombieData, ice_time: &[i64], time: i64, animation: Option<&Vec<Num>>) -> (f64, f64) {
    let pool = rayon::ThreadPoolBuilder::new().stack_size(16 << 20).build().unwrap();
    let walk_time_ = walk_time(data, ice_time, time);
    let x_min = pool.install(|| calculate_animation_impl(data, &walk_time_.0, animation)).0;
    let x_max = pool.install(|| calculate_animation_impl(data, &walk_time_.1, animation)).1;
    (x_min, x_max)
}

fn calculate_regular(data: &ZombieData, ice_time: &[i64], time: i64) -> (f64, f64) {
    let MovementType::Regular(anim1, anim2) = &data.movement_type else {
        unreachable!();
    };
    let (x_min1, x_max1) = calculate_animation(data, ice_time, time, Some(anim1));
    let (x_min2, x_max2) = calculate_animation(data, ice_time, time, Some(anim2));
    (f64::min(x_min1, x_min2), f64::max(x_max1, x_max2))
}

fn calculate_dancing(data: &ZombieData, ice_time: &[i64], time: i64) -> (f64, f64) {
    let walk_time_ = walk_time(data, ice_time, time).0;
    let t = if walk_time_[0].1 { 0 } else { walk_time_[0].0 };
    let x_min = calculate_animation(data, &[], min(t, 310), None).0;
    let x_max = calculate_animation(data, &[], max(t, 299), None).1;
    (x_min, x_max)
}

fn calculate_zomboni(data: &ZombieData, time: i64) -> (f64, f64) {
    let (mut x_min, mut x_max) = (data.spawn.0 as f64, data.spawn.1 as f64);
    for _ in 0..time {
        x_min -= ((x_min - 700.0).floor() / 2000.0 + 0.25).clamp(0.1, 0.25);
        x_max -= ((x_max - 700.0).floor() / 2000.0 + 0.25).clamp(0.1, 0.25);
    }
    (x_min, x_max)
}

#[must_use]
pub fn calculate_extrema(zombie: ZombieType, ice_time: &[i64], time: i64) -> (f64, f64) {
    let data = &crate::parse_data::ZOMBIE_DB[&zombie];
    match data.movement_type {
        MovementType::Constant => calculate_constant(data, ice_time, time),
        MovementType::Animation(_) => calculate_animation(data, ice_time, time, None),
        MovementType::Regular(_, _) => calculate_regular(data, ice_time, time),
        MovementType::DanceCheat => unimplemented!(),
        MovementType::Dancing(_) => calculate_dancing(data, ice_time, time),
        MovementType::Zomboni => calculate_zomboni(data, time),
    }
}
